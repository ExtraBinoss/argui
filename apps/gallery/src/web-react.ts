import { mountGallery } from './react/main'
import { mountWebGallery } from './web-mount'

mountWebGallery('argui-root', mountGallery).catch(error => {
  const root = document.getElementById('argui-root')
  if (root) root.textContent = `Argui could not start: ${String(error)}`
  console.error(error)
})
