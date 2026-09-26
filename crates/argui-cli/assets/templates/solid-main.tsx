import { NativeHost, type NativeBridge, type NativeNode } from '@argui/host'
import { createSignal, render, useNativeHost } from '@argui/solid'

function App() {
  const [count, setCount] = createSignal(0)
  return <column width="100%" height="100%" padding={32} gap={16}>
    <text fontSize={30}>Hello from Argui</text>
    <text>{`Count: ${count()}`}</text>
    <rectangle width={180} height={48} background="#2563eb" onClick={() => setCount(count() + 1)}>
      <text color="#ffffff">Increment</text>
    </rectangle>
  </column>
}

/** Mounts the shared Solid scene into a native or Web Argui bridge. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', '100%')
  host.setProperty(root, 'height', '100%')
  const dispose = render(() => <App /> as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); host.dispose() }
}
