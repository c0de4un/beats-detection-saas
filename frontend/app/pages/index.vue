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

      <!-- Карточка загрузки файла (только для авторизованных) -->
      <div v-if="authStore.isAuthenticated" class="bg-white p-6 sm:p-8 rounded-xl shadow-sm border border-gray-200 space-y-6">
        <h2 class="text-xl font-semibold text-gray-800">Загрузка аудио для анализа</h2>

        <div class="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-gray-300 border-dashed rounded-md">
          <div class="space-y-1 text-center">
            <svg class="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48" aria-hidden="true">
              <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H8a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28" />
            </svg>
            <div class="flex text-sm text-gray-600">
              <label for="file-upload" class="relative cursor-pointer bg-white rounded-md font-medium text-indigo-600 hover:text-indigo-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500">
                <span>Загрузить файл</span>
                <input id="file-upload" name="file-upload" type="file" class="sr-only" accept=".mp3,.wav,.ogg" @change="onFileSelected" />
              </label>
              <p class="pl-1">или перетащите его сюда</p>
            </div>
            <p class="text-xs text-gray-500">MP3, WAV, OGG до 10MB</p>
          </div>
        </div>

        <!-- Индикация загрузки -->
        <div v-if="isUploading" class="text-center text-sm text-gray-600">
          Обработка файла на Rust... <span class="animate-pulse">⏳</span>
        </div>

        <!-- Вывод результата (заглушка/превью) -->
        <div v-if="uploadResult" class="bg-green-50 p-4 rounded-md border border-green-200">
          <p class="font-medium text-green-800">Анализ завершен!</p>
          <p class="text-sm text-green-700 mt-1">BPM: {{ uploadResult.analysis.bpm }}</p>
          <p class="text-sm text-green-700">Найдено битов: {{ uploadResult.analysis.beats_ms.length }}</p>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'

const authStore = useAuthStore()
const isUploading = ref(false)
const uploadResult = ref<any>(null)

const onFileSelected = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]

  if (!file) return

  isUploading.value = true
  uploadResult.value = null

  try {
    const formData = new FormData()
    formData.append('file', file)

    const response = await $fetch('/api/audio/upload', {
      method: 'POST',
      body: formData,
    })

    uploadResult.value = response
  } catch (error: any) {
    console.error('Upload error:', error)
    alert('Ошибка загрузки файла: ' + (error.data?.message || 'Неизвестная ошибка'))
  } finally {
    isUploading.value = false
  }
}
</script>