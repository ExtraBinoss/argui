import { NativeHost, type NativeBridge, type NativeNode } from '@argui/host'
import { createSignal, render, useNativeHost } from '@argui/solid'

function App() {
  const [count, setCount] = createSignal(0)
  return <column width="fill" height="fill" padding={32} gap={16}>
    <text text="Hello from Argui" font_size={30} />
    <text text={`Count: ${count()}`} />
    <rectangle width={180} height={48} background="#2563eb" onClick={() => setCount(count() + 1)}>
      <text text="Increment" color="#ffffff" />
    </rectangle>
  </column>
}

/** Mounts the shared Solid scene into a native or Web Argui bridge. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', 'fill')
  host.setProperty(root, 'height', 'fill')
  const dispose = render(() => <App /> as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); host.dispose() }
}
