export default defineNuxtPlugin(() => {
    const config = useRuntimeConfig()

    globalThis.$fetch = $fetch.create({
        baseURL: config.public.apiBase,
        onRequest({ options }) {
            const token = useCookie('jwt_token').value

            if (token) {
                options.headers = options.headers || new Headers()

                const headers = options.headers instanceof Headers
                    ? options.headers
                    : new Headers(options.headers as any)

                headers.set('Authorization', `Bearer ${token}`)
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
})