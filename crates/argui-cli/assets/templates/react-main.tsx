/** @jsxImportSource @argui/react */
import { NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)
  return <column width="100%" height="100%" padding={32} gap={16}>
    <text fontSize={30}>Hello from Argui</text>
    <text>{`Count: ${count}`}</text>
    <rectangle width={180} height={48} background="#2563eb" onClick={() => setCount(value => value + 1)}>
      <text color="#ffffff">Increment</text>
    </rectangle>
  </column>
}

/** Mounts the shared React scene into a native or Web Argui bridge. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const root = createRoot(host, 'Column', { width: '100%', height: '100%' })
  root.render(<App />)
  return () => root.unmount()
}
