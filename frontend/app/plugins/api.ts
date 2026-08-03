export default defineNuxtPlugin((nuxtApp) => {
    const config = useRuntimeConfig()
    const token = useCookie<string | null>('jwt_token')

    const api = $fetch.create({
        baseURL: config.public.apiBase,
        onRequest({ options }) {
            if (token.value) {
                const headers = new Headers(options.headers as HeadersInit)
                headers.set('Authorization', `Bearer ${token.value}`)
                options.headers = headers
            }
        },
        onResponseError({ response }) {
            if (response.status === 401) {
                const authStore = useAuthStore()
                authStore.logout()
            }
        }
    })

    return {
        provide: {
            api
        }
    }
})