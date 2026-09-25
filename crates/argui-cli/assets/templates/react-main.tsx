/** @jsxImportSource @argui/react */
import { NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)
  return <column width="fill" height="fill" padding={32} gap={16}>
    <text text="Hello from Argui" font_size={30} />
    <text text={`Count: ${count}`} />
    <rectangle width={180} height={48} background="#2563eb" onClick={() => setCount(value => value + 1)}>
      <text text="Increment" color="#ffffff" />
    </rectangle>
  </column>
}

/** Mounts the shared React scene into a native or Web Argui bridge. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const root = createRoot(host, 'Column', { width: 'fill', height: 'fill' })
  root.render(<App />)
  return () => root.unmount()
}
