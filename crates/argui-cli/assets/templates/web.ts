import { mountArgui } from './mount'

mountArgui('argui-root').catch(error => {
  const root = document.getElementById('argui-root')
  if (root) root.textContent = `Argui could not start: ${String(error)}`
  console.error(error)
})
