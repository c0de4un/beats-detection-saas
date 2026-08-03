use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use serde::Serialize;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::TrackType;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

#[derive(Debug, Serialize)]
pub struct BeatResult {
    pub bpm: f32,
    pub beats_ms: Vec<u64>,
}

pub struct AudioAnalyzer;

impl AudioAnalyzer {
    pub fn analyze(file_path: &str) -> Result<BeatResult, String> {
        // ═════════════════════════════════════════════════════════════════
        // 1. Открываем файл и создаём MediaSourceStream
        // ═════════════════════════════════════════════════════════════════
        let src = File::open(file_path).map_err(|e| e.to_string())?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = Path::new(file_path).extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
            }
        }

        // ═════════════════════════════════════════════════════════════════
        // 2. Probe формата
        // ═════════════════════════════════════════════════════════════════
        let mut probed = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
            .map_err(|e| format!("Format error: {}", e))?;

        // ═════════════════════════════════════════════════════════════════
        // 3. Находим первую аудиодорожку
        // ═════════════════════════════════════════════════════════════════
        let track = probed
            .default_track(TrackType::Audio)
            .ok_or("No valid audio track found")?;

        let track_id = track.id;

        let audio_params = track
            .codec_params
            .as_ref()
            .ok_or("Missing codec params")?
            .audio()
            .ok_or("Not an audio track")?;

        let sample_rate = audio_params
            .sample_rate
            .ok_or("Unknown sample rate")? as f32;

        // ═════════════════════════════════════════════════════════════════
        // 4. Создаём декодер
        // ═════════════════════════════════════════════════════════════════
        let mut decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
            .map_err(|e| format!("Decoder error: {}", e))?;

        let mut interleaved: Vec<f32> = Vec::new();

        // ИСПРАВЛЕНИЕ №1 (главный баг):
        // Раньше здесь было `audio_params.channels.as_slice().len()`.
        // `channels` в `AudioCodecParameters` — это `Option<Channels>`, а `Option::as_slice()` —
        // это метод самого `Option` (даёт срез длины 0 или 1 в зависимости от `None`/`Some`,
        // и не имеет отношения к числу каналов внутри `Channels`). В результате для ЛЮБОГО
        // `Some(_)` вы всегда получали `channels == 1`, даже для стерео-файла.
        // Это тихо (без ошибки компиляции) превращало interleaved-буфер `[L0, R0, L1, R1, ...]`
        // в "моно"-сигнал вдвое длиннее настоящего — отсюда и вдвое растянутая по времени
        // дорожка, вдвое (и больше) БПМ, и куча лишних "битов" на стыках L/R-сэмплов.
        //
        // Берём число каналов из РЕАЛЬНО декодированного буфера (`audio_buf.spec()`) — это
        // гарантированно совпадает с раскладкой данных, которые отдаёт `copy_to_slice_interleaved`.
        let mut channels: Option<usize> = None;

        // ═════════════════════════════════════════════════════════════════
        // 5. Декодируем в interleaved f32
        // ═════════════════════════════════════════════════════════════════
        loop {
            let packet = match probed.next_packet() {
                Ok(Some(p)) => p,
                Ok(None) => break,
                Err(SymphoniaError::IoError(_)) => break, // EOF
                Err(e) => return Err(format!("Read error: {}", e)),
            };

            if packet.track_id != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    channels.get_or_insert_with(|| audio_buf.spec().channels().count().max(1));

                    let n = audio_buf.samples_interleaved();
                    let start = interleaved.len();
                    interleaved.resize(start + n, 0.0);
                    audio_buf.copy_to_slice_interleaved(&mut interleaved[start..]);
                }
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(_) => break,
            }
        }

        let channels = channels.ok_or("No audio samples decoded")?;

        if interleaved.is_empty() {
            return Err("No audio samples decoded".to_string());
        }

        // ═════════════════════════════════════════════════════════════════
        // 6. Interleaved → Mono + Peak Normalization
        // ═════════════════════════════════════════════════════════════════
        let mut samples = Vec::with_capacity(interleaved.len() / channels);
        for frame in interleaved.chunks_exact(channels) {
            samples.push(frame.iter().sum::<f32>() / channels as f32);
        }

        let max_amp = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if max_amp > 0.0 {
            for s in &mut samples {
                *s /= max_amp;
            }
        }

        // ═════════════════════════════════════════════════════════════════
        // 7. Beat Detection: Energy (RMS) + Spectral Flux
        // ═════════════════════════════════════════════════════════════════
        const FRAME_SIZE: usize = 1024;
        const HOP_SIZE: usize = 512; // 50 % overlap

        let mut energies: Vec<f32> = Vec::new();
        let mut flux_values: Vec<f32> = Vec::new();

        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(FRAME_SIZE);

        let mut prev_magnitudes: Option<Vec<f32>> = None;

        for window in samples.windows(FRAME_SIZE).step_by(HOP_SIZE) {
            if window.len() < FRAME_SIZE {
                break;
            }

            // --- RMS Energy ---
            let rms = (window.iter().map(|s| s * s).sum::<f32>() / FRAME_SIZE as f32).sqrt();
            energies.push(rms);

            // --- Spectral Flux ---
            let mut buf: Vec<Complex<f32>> = window.iter().map(|&s| Complex::new(s, 0.0)).collect();

            // Окно Ханна
            for (i, c) in buf.iter_mut().enumerate() {
                let w = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (FRAME_SIZE - 1) as f32).cos();
                c.re *= w;
            }

            fft.process(&mut buf);

            let mags: Vec<f32> = buf[..FRAME_SIZE / 2].iter().map(|c| c.norm()).collect();

            let flux = if let Some(prev) = &prev_magnitudes {
                mags.iter()
                    .zip(prev.iter())
                    .map(|(c, p)| (c - p).max(0.0))
                    .sum::<f32>()
            } else {
                0.0
            };

            flux_values.push(flux);
            prev_magnitudes = Some(mags);
        }

        if energies.is_empty() {
            return Err("Not enough audio data for analysis".to_string());
        }

        // ═════════════════════════════════════════════════════════════════
        // 8. Комбинированный score + сглаживание
        // ═════════════════════════════════════════════════════════════════
        let max_e = energies.iter().copied().fold(0.0f32, f32::max).max(1e-10);
        let max_f = flux_values.iter().copied().fold(0.0f32, f32::max).max(1e-10);

        let mut score: Vec<f32> = energies
            .iter()
            .zip(flux_values.iter())
            .map(|(e, f)| (e / max_e) * 0.35 + (f / max_f) * 0.65)
            .collect();

        // Moving average (3 кадра)
        const SMOOTH: usize = 3;
        if score.len() >= SMOOTH {
            score = score
                .windows(SMOOTH)
                .map(|w| w.iter().sum::<f32>() / SMOOTH as f32)
                .collect();
        }

        // ═════════════════════════════════════════════════════════════════
        // 9. Кандидаты в онсеты (адаптивный порог) — ПОЛНЫЙ, плотный список.
        //    Это ещё не финальные "биты" для монтажа: этот список нужен только как
        //    статистически надёжная основа для расчёта BPM (гистограмме нужно много
        //    интервалов, а не 3-7 точек).
        // ═════════════════════════════════════════════════════════════════
        const ONSET_WINDOW: usize = 10;
        const THRESHOLD_MUL: f32 = 1.35;
        const MIN_CANDIDATE_MS: f32 = 120.0; // только чтобы не дублировать соседние кадры

        // (время в мс, "сила" пика — насколько он выделяется над своим окружением)
        let mut onset_candidates: Vec<(u64, f32)> = Vec::new();

        for i in ONSET_WINDOW..score.len() {
            let local = &score[i - ONSET_WINDOW..i];
            let avg = local.iter().sum::<f32>() / ONSET_WINDOW as f32;
            let std = (local.iter().map(|s| (s - avg).powi(2)).sum::<f32>() / ONSET_WINDOW as f32).sqrt();

            let threshold = avg + THRESHOLD_MUL * std;

            if score[i] > threshold && score[i] > 0.05 {
                let frame_idx = i + (SMOOTH / 2);
                let time_ms = ((frame_idx * HOP_SIZE) as f32 / sample_rate * 1000.0) as u64;
                let strength = score[i] - threshold; // насколько сильно превышен порог

                let ok = onset_candidates
                    .last()
                    .map_or(true, |&(last_ms, _)| time_ms.saturating_sub(last_ms) as f32 > MIN_CANDIDATE_MS);

                if ok {
                    onset_candidates.push((time_ms, strength));
                }
            }
        }

        if onset_candidates.is_empty() {
            return Err("No onsets detected".to_string());
        }

        // BPM считаем по ПОЛНОМУ списку кандидатов — так гистограмма интервалов
        // остаётся статистически осмысленной, независимо от того, сколько маркеров
        // мы дальше покажем пользователю.
        let raw_times: Vec<u64> = onset_candidates.iter().map(|&(t, _)| t).collect();
        let bpm = Self::estimate_bpm(&raw_times);

        // ═════════════════════════════════════════════════════════════════
        // 10. ИСПРАВЛЕНИЕ №2:
        //     Из плотного списка онсетов выбираем только 3-7 САМЫХ СИЛЬНЫХ и хорошо
        //     разнесённых по времени — жадный non-max suppression по убыванию "силы" пика.
        //     Это надёжнее, чем просто поднять THRESHOLD_MUL: фиксированный порог даёт
        //     непредсказуемое число битов в зависимости от жанра/громкости трека, а
        //     "top-N с минимальным интервалом" всегда возвращает управляемое количество.
        // ═════════════════════════════════════════════════════════════════
        const TARGET_MIN: usize = 3;
        const TARGET_MAX: usize = 5;

        let total_duration_ms = raw_times.last().copied().unwrap_or(1) as f32;
        let min_gap_ms = (total_duration_ms / (TARGET_MAX as f32 + 1.0)).max(300.0);

        let beats_ms = Self::pick_salient_beats(&onset_candidates, min_gap_ms, TARGET_MAX);

        // Если из-за большого min_gap_ms отобралось меньше TARGET_MIN — один раз ослабляем
        // интервал и пробуем снова. Если сильных онсетов физически меньше TARGET_MIN,
        // возвращаем то, что реально есть, а не выдумываем лишние биты.
        let beats_ms = if beats_ms.len() < TARGET_MIN && onset_candidates.len() >= TARGET_MIN {
            Self::pick_salient_beats(&onset_candidates, min_gap_ms / 2.0, TARGET_MAX)
        } else {
            beats_ms
        };

        Ok(BeatResult { bpm, beats_ms })
    }

    /// Жадный отбор до `max_count` самых сильных пиков с минимальным интервалом
    /// `min_gap_ms` между ними (non-max suppression). Возвращает времена в
    /// хронологическом порядке.
    fn pick_salient_beats(candidates: &[(u64, f32)], min_gap_ms: f32, max_count: usize) -> Vec<u64> {
        let mut ranked: Vec<(u64, f32)> = candidates.to_vec();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut selected: Vec<u64> = Vec::new();
        for &(time_ms, _) in &ranked {
            if selected.len() >= max_count {
                break;
            }
            let far_enough = selected
                .iter()
                .all(|&b| (time_ms as i64 - b as i64).unsigned_abs() as f32 > min_gap_ms);
            if far_enough {
                selected.push(time_ms);
            }
        }

        selected.sort_unstable();
        selected
    }

    fn estimate_bpm(beats_ms: &[u64]) -> f32 {
        if beats_ms.len() < 2 {
            return 0.0;
        }

        let intervals: Vec<f32> = beats_ms.windows(2).map(|w| (w[1] - w[0]) as f32).collect();

        // Гистограмма с бинами по 50 мс
        let mut hist: HashMap<u64, usize> = HashMap::new();
        for &iv in &intervals {
            let bin = (iv as u64 / 50) * 50;
            *hist.entry(bin).or_insert(0) += 1;
        }

        let mode = hist
            .iter()
            .max_by_key(|&(_, c)| c)
            .map(|(iv, _)| *iv as f32)
            .unwrap_or_else(|| {
                let mut s = intervals.clone();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap());
                s[s.len() / 2]
            });

        if mode > 0.0 {
            60000.0 / mode
        } else {
            0.0
        }
    }
}