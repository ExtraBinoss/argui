/** @jsxImportSource @argui/react */
import { ApplicationServices, createThemeRuntime, NativeHost, type NativeBridge } from '@argui/host'
import { createRoot, ThemeProvider } from '@argui/react'
import { widgetThemeDefinition, type WidgetTheme } from '@argui/widgets/react'
import { Gallery } from './gallery'

/** Mounts the React gallery into a native or browser Argui host. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string, browser = false): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const runtime = createThemeRuntime<WidgetTheme>(bridge, widgetThemeDefinition)
  const services = new ApplicationServices(bridge)
  const root = createRoot(host, 'column', { width: '100%', height: '100%' })
  root.render(
    <ThemeProvider runtime={runtime}>
      <Gallery runtime={runtime} services={services} browser={browser} />
    </ThemeProvider>,
  )
  return () => { root.unmount(); services.dispose(); runtime.dispose() }
}

export { mountGallery as mountReactGallery }
