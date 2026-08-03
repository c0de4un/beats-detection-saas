<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-100 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 bg-white p-8 rounded-xl shadow-lg border border-gray-200">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
          {{ isLogin ? 'Вход в систему' : 'Регистрация' }}
        </h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          Korgi.Beats Detection
        </p>
      </div>

      <Form @submit="onSubmit" :validation-schema="schema" v-slot="{ errors, isSubmitting }" class="mt-8 space-y-6">

        <!-- Email -->
        <div>
          <label for="email" class="block text-sm font-medium text-gray-700">Email</label>
          <div class="mt-1">
            <Field
                id="email"
                name="email"
                type="email"
                placeholder="test@yandex.ru"
                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                :class="{ 'border-red-500': errors.email }"
            />
            <ErrorMessage name="email" class="text-red-500 text-xs mt-1" />
          </div>
        </div>

        <!-- Пароль -->
        <div>
          <label for="password" class="block text-sm font-medium text-gray-700">Пароль</label>
          <div class="mt-1">
            <Field
                id="password"
                name="password"
                type="password"
                placeholder="••••••••"
                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                :class="{ 'border-red-500': errors.password }"
            />
            <ErrorMessage name="password" class="text-red-500 text-xs mt-1" />
          </div>
        </div>

        <!-- Ошибка от API -->
        <div v-if="apiError" class="text-red-500 text-sm text-center bg-red-50 p-2 rounded-md border border-red-200">
          {{ apiError }}
        </div>

        <!-- Кнопка отправки -->
        <div>
          <button
              type="submit"
              :disabled="isSubmitting"
              class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 transition-colors"
          >
            {{ isSubmitting ? 'Загрузка...' : (isLogin ? 'Войти' : 'Зарегистрироваться') }}
          </button>
        </div>

        <!-- Переключатель режима -->
        <div class="text-center text-sm">
          <button type="button" @click="isLogin = !isLogin" class="text-indigo-600 hover:text-indigo-500 font-medium">
            {{ isLogin ? 'Нет аккаунта? Зарегистрироваться' : 'Уже есть аккаунта? Войти' }}
          </button>
        </div>
      </Form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Form, Field, ErrorMessage } from 'vee-validate'
import * as yup from 'yup'
import { useAuthStore } from '~/stores/auth'

const isLogin = ref(true)
const apiError = ref<string | null>(null)
const authStore = useAuthStore()
const router = useRouter()
const { $api } = useNuxtApp()

const schema = yup.object({
  email: yup.string().email('Некорректный email').required('Обязательное поле'),
  password: yup.string().min(6, 'Минимум 6 символов').required('Обязательное поле'),
})

const onSubmit = async (values: any) => {
  apiError.value = null
  try {
    const endpoint = isLogin.value ? '/auth/login' : '/auth/register'

    // Используем $api
    const response = await $api(endpoint, {
      method: 'POST',
      body: {
        email: values.email,
        password: values.password
      },
    })

    authStore.setAuth(response.token, response.user)

    await router.push('/')
  } catch (error: any) {
    if (error.data?.errors) {
      const firstErrorKey = Object.keys(error.data.errors)[0]
      apiError.value = error.data.errors[firstErrorKey]
    } else if (error.data?.message) {
      apiError.value = error.data.message
    } else {
      apiError.value = 'Произошла ошибка сети. Попробуйте позже.'
    }
  }
}
</script>