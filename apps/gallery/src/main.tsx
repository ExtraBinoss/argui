import { NativeHost, type NativeBridge, type NativeNode } from '@argui/host'
import { render, useNativeHost } from '@argui/solid'
import { Gallery } from './gallery'

/** Mounts the Solid gallery into one canonical native Argui tree. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', 'fill')
  host.setProperty(root, 'height', 'fill')
  const dispose = render(() => <Gallery /> as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); host.dispose() }
}
