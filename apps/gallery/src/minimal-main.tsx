import { NativeHost, type NativeBridge, type NativeNode } from '@argui/host'
import { render, useNativeHost } from '@argui/solid'

/** Mounts one TSX text node to measure the native runtime's baseline memory. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', 'fill')
  host.setProperty(root, 'height', 'fill')
  const dispose = render(() => <text text="Bonjour Argui" color="#ffffff" font_size={24} /> as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); host.dispose() }
}
