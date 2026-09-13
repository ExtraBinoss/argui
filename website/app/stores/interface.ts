export const useInterfaceStore = defineStore('interface', () => {
  const query = ref('')
  const navigationOpen = ref(false)
  const theme = ref<'light' | 'dark'>('light')

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    const preference = useCookie('argui-theme', { maxAge: 31_536_000, sameSite: 'lax' })
    preference.value = theme.value
  }

  return { query, navigationOpen, theme, toggleTheme }
})
