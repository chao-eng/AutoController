import { ref } from 'vue'

type Theme = 'dark' | 'light'

const currentTheme = ref<Theme>('dark')

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    document.documentElement.setAttribute('data-theme', theme)
  }

  function toggleTheme() {
    setTheme(currentTheme.value === 'dark' ? 'light' : 'dark')
  }

  return {
    currentTheme,
    setTheme,
    toggleTheme,
  }
}
