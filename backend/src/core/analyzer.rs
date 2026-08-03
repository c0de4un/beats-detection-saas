use std::fs::File;

use serde::Serialize;
use symphonia::core::audio::sample::Sample;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

#[derive(Debug, Serialize)]
pub struct BeatResult {
    pub bpm: f32,
    pub beats_ms: Vec<u64>,
}

pub struct AudioAnalyzer;

impl AudioAnalyzer {
    pub fn analyze(file_path: &str) -> Result<BeatResult, String> {
        let src = File::open(file_path).map_err(|e| e.to_string())?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = std::path::Path::new(file_path).extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
            }
        }

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
            .map_err(|e| format!("Format error: {}", e))?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or("No valid audio track found")?;

        let track_id = track.id;
        let codec_params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or("Unsupported or missing audio codec parameters")?
            .clone();

        let sample_rate = codec_params.sample_rate.ok_or("Unknown sample rate")? as f32;

        let mut decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&codec_params, &AudioDecoderOptions::default())
            .map_err(|e| format!("Decoder error: {}", e))?;

        let mut samples: Vec<f32> = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => break, // Конец потока
                Err(SymphoniaError::IoError(_)) => break,
                Err(e) => return Err(format!("Read error: {}", e)),
            };

            if packet.track_id != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let channels = audio_buf.spec().channels().count().max(1);

                    let mut interleaved = vec![f32::MID; audio_buf.samples_interleaved()];
                    audio_buf.copy_to_slice_interleaved(&mut interleaved);

                    for frame in interleaved.chunks(channels) {
                        let mono = frame.iter().sum::<f32>() / channels as f32;
                        samples.push(mono);
                    }
                }
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(_) => break,
            }
        }

        let frame_size = 1024;
        let mut beats_ms = Vec::new();
        let mut energies = Vec::new();

        for chunk in samples.chunks(frame_size) {
            let energy: f32 = chunk.iter().map(|s| s * s).sum::<f32>() / frame_size as f32;
            energies.push(energy);
        }

        let window_size = 20;
        for i in window_size..energies.len() {
            let avg: f32 = energies[i - window_size..i].iter().sum::<f32>() / window_size as f32;

            if energies[i] > avg * 1.5 && energies[i] > 0.01 {
                let frame_time_ms = ((i * frame_size) as f32 / sample_rate * 1000.0) as u64;

                if beats_ms.last().map_or(true, |&t| frame_time_ms - t > 200) {
                    beats_ms.push(frame_time_ms);
                }
            }
        }

        let bpm = if beats_ms.len() < 2 {
            0.0
        } else {
            let intervals: Vec<f32> = beats_ms.windows(2)
                .map(|w| (w[1] - w[0]) as f32)
                .collect();
            let avg_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
            if avg_interval > 0.0 { 60000.0 / avg_interval } else { 0.0 }
        };

        Ok(BeatResult { bpm, beats_ms })
    }
}