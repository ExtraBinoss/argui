/** @jsxImportSource @argui/react */
import { createThemeRuntime, NativeHost, type NativeBridge } from '@argui/host'
import { createRoot, ThemeProvider } from '@argui/react'
import { widgetThemeDefinition, type WidgetTheme } from '@argui/widgets/react'
import { Gallery } from './gallery'

/** Mounts the five-widget React gallery into a native Argui host. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const runtime = createThemeRuntime<WidgetTheme>(bridge, widgetThemeDefinition)
  const root = createRoot(host, 'column', { width: '100%', height: '100%' })
  root.render(
    <ThemeProvider runtime={runtime}>
      <Gallery runtime={runtime} />
    </ThemeProvider>,
  )
  return () => { root.unmount(); runtime.dispose() }
}

export { mountGallery as mountReactGallery }
