import { NativeHost, createThemeRuntime, type NativeBridge, type NativeNode } from '@argui/host'
import { ThemeProvider, render, useNativeHost } from '@argui/solid'
import { widgetThemeDefinition, type WidgetTheme } from '@argui/widgets/solid'
import { Gallery } from './gallery'

/** Mounts the five-widget Solid gallery into a native Argui host. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const runtime = createThemeRuntime<WidgetTheme>(bridge, widgetThemeDefinition)
  useNativeHost(host)
  const root = host.createElement('column')
  host.setProperty(root, 'width', '100%')
  host.setProperty(root, 'height', '100%')
  const dispose = render(() => (
    <ThemeProvider runtime={runtime}>
      <Gallery runtime={runtime} />
    </ThemeProvider>
  ) as unknown as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); runtime.dispose(); host.dispose() }
}
