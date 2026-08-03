export default defineNuxtRouteMiddleware((to) => {
    const authStore = useAuthStore()

    const publicRoutes = ['/auth']

    if (!authStore.isAuthenticated && !publicRoutes.includes(to.path)) {
        return navigateTo('/auth')
    }

    if (authStore.isAuthenticated && to.path === '/auth') {
        return navigateTo('/')
    }
})