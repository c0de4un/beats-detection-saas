export default defineNuxtConfig({
  compatibilityDate: '2026-07-30', // <- Убирает warning B5001
  devtools: { enabled: true },
  modules: [
    '@nuxtjs/tailwindcss',
    '@pinia/nuxt'
  ],

  routeRules: {
    '/api/**': {
      proxy: 'http://localhost:80/api/**'
    }
  },

  runtimeConfig: {
    public: {
      apiBase: '/api'
    }
  }
})