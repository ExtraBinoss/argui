import { mountGallery } from './solid/main'
import { mountWebGallery } from './web-mount'

mountWebGallery('argui-root', mountGallery).catch(error => {
  const root = document.getElementById('argui-root')
  if (root) root.textContent = `Argui could not start: ${String(error)}`
  window.parent.postMessage({ type: 'argui:error', message: String(error) }, window.location.origin)
  console.error(error)
})
