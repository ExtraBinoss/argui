import { createSignal, onMount } from 'solid-js'
import type { ApplicationServices } from '@argui/host'
import { useTheme } from '@argui/solid'
import { Button, type WidgetTheme } from '@argui/widgets/solid'

/** Exercises a monitor-sized transparent X11 window with a native input hole. */
export function ScreenSpotlightPage(props: { services: ApplicationServices }) {
  const theme = useTheme<WidgetTheme>()
  const [backend, setBackend] = createSignal('Checking the desktop backend…')
  const [supported, setSupported] = createSignal(false)
  const [status, setStatus] = createSignal('')
  onMount(() => {
    void props.services.getWindowInfo().then((info) => {
      const capabilities = info.capabilities
      const available = capabilities.backend === 'x11' && capabilities.absolutePosition && capabilities.inputRegions && capabilities.transparentCompositing
      setSupported(available)
      setBackend(`${capabilities.backend.toUpperCase()} · ${available ? 'spotlight available' : 'spotlight requires X11, absolute positioning, input regions, and transparency'} · zoom ${Math.round(info.uiZoomFactor * 100)}%`)
    }).catch((error: unknown) => setBackend(String(error)))
  })
  const run = async (operation: () => Promise<void>) => {
    try { await operation(); setStatus('Applied.') }
    catch (error) { setStatus(String(error)) }
  }
  return <column id="screen-spotlight-page" width="100%" gap={12}>
    <text color={theme().text} fontSize={24}>Screen spotlight</text>
    <text color={theme().textMuted}>A transparent, monitor-sized X11 overlay dims the screen around a clear selection. Drag on the dim area to redraw it. Click inside the clear area to reach the app underneath. Right-click the dim area or press Escape while the overlay is focused to close.</text>
    <text color={theme().textMuted}>{backend()}</text>
    <row gap={8} wrap={true}>
      <Button id="spotlight-open" disabled={!supported()} onClick={() => void run(() => props.services.call<void>('windows', 'openSpotlight'))}>Open spotlight</Button>
      <Button id="spotlight-passthrough" disabled={!supported()} variant="outline" onClick={() => void run(() => props.services.setWindowMousePassthrough('spotlight', true))}>Pass through all clicks</Button>
      <Button id="spotlight-close" disabled={!supported()} variant="outline" onClick={() => void run(() => props.services.closeWindow('spotlight'))}>Close spotlight</Button>
    </row>
    <text id="spotlight-status" color={theme().text}>{status()}</text>
  </column>
}
