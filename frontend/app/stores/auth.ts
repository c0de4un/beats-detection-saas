import { defineStore } from 'pinia'

interface User {
    id: string
    email: string
    created_at?: string
}

export const useAuthStore = defineStore('auth', () => {
    const token = useCookie<string | null>('jwt_token')
    const user = ref<User | null>(null)

    const isAuthenticated = computed(() => !!token.value)

    function setAuth(tokenValue: string, userData: User) {
        token.value = tokenValue
        user.value = userData
    }

    function logout() {
        token.value = null
        user.value = null
        navigateTo('/auth')
    }

    return { user, token, isAuthenticated, setAuth, logout }
})