<!-- app/pages/index.vue -->
<template>
  <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
    <div class="w-full max-w-2xl">

      <!-- Шапка -->
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold text-gray-900">Korgi.Beats Engine</h1>
        <div v-if="authStore.isAuthenticated" class="flex items-center space-x-4">
          <span class="text-sm text-gray-600">{{ authStore.user?.email }}</span>
          <button @click="authStore.logout()" class="text-sm text-indigo-600 hover:underline">
            Выйти
          </button>
        </div>
      </div>

      <!-- КАРТОЧКА РЕЗУЛЬТАТА (Появляется над формой) -->
      <div v-if="resultData" class="bg-white p-6 sm:p-8 rounded-xl shadow-sm border border-gray-200 mb-6">
        <h2 class="text-xl font-semibold text-gray-800 mb-4 truncate" :title="resultData.original_name">
          {{ resultData.original_name }}
        </h2>

        <AudioWaveform
            :audioUrl="localAudioUrl"
            :beats="resultData.beats"
            :bpm="resultData.bpm"
        />
      </div>

      <!-- КАРТОЧКА ЗАГРУЗКИ ФАЙЛА -->
      <div v-if="authStore.isAuthenticated" class="bg-white p-6 sm:p-8 rounded-xl shadow-sm border border-gray-200 space-y-6">
        <h2 class="text-xl font-semibold text-gray-800">Загрузка аудио для анализа</h2>

        <div
            class="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-gray-300 border-dashed rounded-md transition-colors"
            :class="{ 'border-indigo-400 bg-indigo-50 opacity-60 pointer-events-none': isProcessing }"
        >
          <div class="space-y-1 text-center">
            <svg v-if="!isProcessing" class="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48" aria-hidden="true">
              <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H8a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28" />
            </svg>

            <!-- Спиннер во время обработки -->
            <div v-else class="flex justify-center items-center py-2">
              <svg class="animate-spin h-10 w-10 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>

            <div class="flex text-sm text-gray-600">
              <label for="file-upload" class="relative cursor-pointer bg-white rounded-md font-medium text-indigo-600 hover:text-indigo-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500">
                <span>Загрузить файл</span>
                <input id="file-upload" name="file-upload" type="file" class="sr-only" accept=".mp3,.wav,.ogg" @change="onFileSelected" :disabled="isProcessing" />
              </label>
              <p class="pl-1">или перетащите его сюда</p>
            </div>
            <p class="text-xs text-gray-500">MP3, WAV, OGG до 10MB</p>
          </div>
        </div>

        <!-- Текстовый индикатор обработки -->
        <div v-if="isProcessing" class="text-center text-sm text-indigo-600 font-medium animate-pulse">
          Обработка файла на Rust... Поиск Down-Beats
        </div>

        <!-- Вывод ошибки -->
        <div v-if="errorMsg" class="bg-red-50 p-4 rounded-md border border-red-200">
          <p class="font-medium text-red-800">Ошибка</p>
          <p class="text-sm text-red-700 mt-1">{{ errorMsg }}</p>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import AudioWaveform from '~/components/AudioWaveform.vue'

const authStore = useAuthStore()
// Получаем наш кастомный инстанс $api
const { $api } = useNuxtApp()

// Строгая типизация ответов API
interface UploadResponse {
  audio_file_id: string
  job_id: string
  message: string
  status: string
}

interface JobResult {
  id: string
  filename: string
  original_name: string
  size: number
  status: string
  bpm: number
  beats: number[]
  created_at: string
}

interface JobResponse {
  job_id: string
  status: string
  result: JobResult
}

// Реактивные состояния
const isProcessing = ref(false)
const resultData = ref<JobResult | null>(null)
const localAudioUrl = ref<string>('')
const errorMsg = ref<string | null>(null)

let pollingTimer: ReturnType<typeof setInterval> | null = null

const onFileSelected = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]

  if (!file) return

  // Сбрасываем прошлые результаты
  resultData.value = null
  errorMsg.value = null
  isProcessing.value = true

  // Создаем локальный URL для аудиоплеера (чтобы не качать файл обратно с бэкенда)
  localAudioUrl.value = URL.createObjectURL(file)

  try {
    const formData = new FormData()
    formData.append('file', file)

    // Используем $api вместо $fetch. Путь начинается с /audio, так как baseURL уже /api
    const uploadRes = await $api<UploadResponse>('/audio/upload', {
      method: 'POST',
      body: formData,
    })

    // Запускаем фоновый опрос статуса задачи
    startPolling(uploadRes.job_id)

  } catch (error: any) {
    console.error('Upload error:', error)
    errorMsg.value = 'Произошла сетевая ошибка. Попробуйте позже'
    isProcessing.value = false
    URL.revokeObjectURL(localAudioUrl.value)
  }
}

const startPolling = (jobId: string) => {
  pollingTimer = setInterval(async () => {
    try {
      const jobRes = await $api<JobResponse>('/jobs', {
        method: 'GET',
        params: { job_id: jobId }
      })

      if (jobRes.status === 'completed') {
        if (pollingTimer) {
          clearInterval(pollingTimer)
          pollingTimer = null
        }

        resultData.value = jobRes.result
        isProcessing.value = false
      } else if (jobRes.status === 'failed') {
        if (pollingTimer) {
          clearInterval(pollingTimer)
          pollingTimer = null
        }
        errorMsg.value = 'Ошибка анализа файла на сервере.'
        isProcessing.value = false
        URL.revokeObjectURL(localAudioUrl.value)
      }
    } catch (err) {
      console.error('Polling error:', err)
    }
  }, 2000)
}

// Обязательно очищаем таймер и память при размонтировании компонента
onBeforeUnmount(() => {
  if (pollingTimer) {
    clearInterval(pollingTimer)
  }
  if (localAudioUrl.value) {
    URL.revokeObjectURL(localAudioUrl.value)
  }
})
</script>