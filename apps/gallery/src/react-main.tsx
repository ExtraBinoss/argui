/** @jsxImportSource @argui/react */
import { NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { ReactGallery } from './react-gallery'

/** Mounts the React gallery through the same Argui native host and schema. */
export function mountReactGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const root = createRoot(host, 'Column', { width: 'fill', height: 'fill' })
  root.render(<ReactGallery />)
  return () => root.unmount()
}

export { mountReactGallery as mountGallery }
