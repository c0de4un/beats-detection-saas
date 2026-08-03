<!-- app/components/AudioWaveform.vue -->
<template>
  <div class="w-full">
    <div ref="waveformContainer" class="w-full"></div>

    <!-- Контролы воспроизведения -->
    <div class="flex items-center gap-4 mt-4">
      <button
          @click="togglePlay"
          class="flex items-center justify-center w-10 h-10 rounded-full bg-indigo-600 text-white hover:bg-indigo-700 transition-colors focus:outline-none"
      >
        <svg v-if="!isPlaying" xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M5.75 4.5a.75.75 0 00-.75.75v9.5a.75.75 0 001.5 0v-9.5a.75.75 0 00-.75-.75zm8.5 0a.75.75 0 00-.75.75v9.5a.75.75 0 001.5 0v-9.5a.75.75 0 00-.75-.75z" />
        </svg>
      </button>

      <div class="text-sm text-gray-600">
        <span class="font-medium text-indigo-600">{{ currentBPM }}</span> BPM
        | Найдено битов: <span class="font-medium">{{ beats.length }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import WaveSurfer from 'wavesurfer.js'
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.esm.js'

const props = defineProps<{
  audioUrl: string
  beats: number[]
  bpm: number
}>()

const waveformContainer = ref<HTMLElement | null>(null)
const isPlaying = ref(false)
let wavesurfer: WaveSurfer | null = null
let regionsPlugin: RegionsPlugin | null = null

const currentBPM = ref(props.bpm)

onMounted(() => {
  if (!waveformContainer.value) return

  // 2. Инициализируем плагин Regions
  regionsPlugin = RegionsPlugin.create()

  wavesurfer = WaveSurfer.create({
    container: waveformContainer.value,
    waveColor: '#94a3b8',
    progressColor: '#4f46e5',
    barWidth: 2,
    barRadius: 1,
    barGap: 2,
    height: 100,
    plugins: [regionsPlugin]
  })

  wavesurfer.load(props.audioUrl)

  wavesurfer.on('ready', () => {
    // Как только аудио загружено, рисуем маркеры
    drawBeats()
  })

  wavesurfer.on('play', () => isPlaying.value = true)
  wavesurfer.on('pause', () => isPlaying.value = false)
  wavesurfer.on('finish', () => isPlaying.value = false)
})

// Рисуем маркеры битов поверх волны с помощью Regions
const drawBeats = () => {
  if (!wavesurfer || !regionsPlugin) return

  const duration = wavesurfer.getDuration()
  if (duration <= 0) return
  regionsPlugin.clearRegions()

  props.beats.forEach((beatMs) => {
    const beatSec = beatMs / 1000

    regionsPlugin.addRegion({
      start: beatSec,
      color: 'rgba(239, 68, 68, 0.6)', // Полупрозрачный красный
      drag: false, // Запрещаем перетаскивание
      resize: false // Запрещаем изменение размера
    })
  })
}

const togglePlay = () => {
  if (!wavesurfer) return
  wavesurfer.playPause()
}

onBeforeUnmount(() => {
  if (wavesurfer) {
    wavesurfer.destroy()
  }
})
</script>

<style scoped>
:deep(div[style*="position: absolute"]) {
  z-index: 5 !important;
}
</style>