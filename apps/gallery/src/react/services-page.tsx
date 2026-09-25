/** @jsxImportSource @argui/react */
import { useEffect, useRef, useState, type ReactElement } from 'react'
import type { ApplicationServices } from '@argui/host'
import { tr } from '@argui/i18n'
import { Button, InputField, type Palette } from '@argui/widgets/react'

/** Exercises the same native service paths from React. */
export function ReactServicesPage(props: { services: ApplicationServices; theme: Palette }): ReactElement {
  const [status, setStatus] = useState('Choose a native service.')
  const [trayEnabled, setTrayEnabled] = useState(true)
  const [closeHides, setCloseHides] = useState(true)
  const [preferredCloseHides, setPreferredCloseHides] = useState(true)
  const [message, setMessage] = useState('Bonjour from the main gallery')
  const [windowTitle, setWindowTitle] = useState('Argui Companion')
  const [windowWidth, setWindowWidth] = useState('480')
  const [windowHeight, setWindowHeight] = useState('260')
  const [targetWindow, setTargetWindow] = useState<'main' | 'companion'>('companion')
  const [decorations, setDecorations] = useState(true)
  const [transparent, setTransparent] = useState(false)
  const [backdrop, setBackdrop] = useState(false)
  const [backdropScope, setBackdropScope] = useState<'window' | 'panel'>('window')
  useEffect(() => props.services.onEvent((event) => {
    if (event.type === 'menu') setStatus(`Tray action: ${event.id}`)
    else if (event.type === 'shortcut' && event.state === 'pressed') setStatus(`Global shortcut: ${event.id}`)
    else if (event.type === 'shortcutError' || event.type === 'trayError') setStatus(`${event.type}: ${event.message}`)
    else if (event.type === 'window') setStatus(`${event.window}: ${event.message}`)
    else if (event.type === 'windowAppearanceError') setStatus(`${event.window} backdrop: ${event.message}`)
  }), [props.services])
  const active = useRef<AbortController | null>(null)
  const run = (name: string, request: (signal: AbortSignal) => Promise<unknown>) => {
    const controller = new AbortController()
    active.current = controller
    setStatus(`${name}: waiting`)
    request(controller.signal).then(
      (value) => setStatus(`${name}: ${value === false ? 'unavailable in this gallery (details below)' : JSON.stringify(value)}`),
      (error: unknown) => setStatus(`${name}: ${error instanceof Error ? error.message : String(error)}`),
    )
  }
  return <column gap={12}>
    <text text="Native application requests" color={props.theme.foreground} font_size={17} />
    <row wrap={true} gap={8}>
      <Button id="service-clipboard-write" label="Copy text" theme={props.theme}
        onClick={() => run('Clipboard write', (signal) => props.services.writeClipboardText('Argui native clipboard', signal))} />
      <Button id="service-clipboard-read" label="Paste text" theme={props.theme}
        onClick={() => run('Clipboard read', (signal) => props.services.readClipboardText(signal))} />
      <Button id="service-file-open" label="Open files" theme={props.theme}
        onClick={() => run('Open files', (signal) => props.services.openFiles({ multiple: true }, signal))} />
      <Button id="service-file-save" label="Save file" theme={props.theme}
        onClick={() => run('Save file', (signal) => props.services.saveFile({ fileName: 'untitled.txt' }, signal))} />
      <Button id="service-menu-capability" label="App menu support" theme={props.theme}
        onClick={() => run('App menu support', () => props.services.supports('menus', 'set'))} />
      <Button id="service-shortcut-capability" label="Global shortcut support" theme={props.theme}
        onClick={() => run('Global shortcut support', () => props.services.supports('shortcuts', 'set'))} />
      <Button id="service-menu-request" label="Set menu" theme={props.theme}
        onClick={() => run('Menu', (signal) => props.services.setMenu([{ id: 'open-services', label: tr('trayOpenServices') }], signal))} />
      <Button id="service-shortcut-request" label="Set shortcut" theme={props.theme}
        onClick={() => run('Shortcut', (signal) => props.services.setGlobalShortcuts([{ id: 'wake', accelerator: 'CmdOrCtrl+KeyH' }], signal))} />
      <Button id="service-close-behavior" label={closeHides ? 'Close: hide' : 'Close: quit'} theme={props.theme}
        onClick={() => {
          const next = !closeHides
          run('Close behavior', (signal) => props.services.setCloseBehavior(next ? 'hide' : 'quit', signal).then(() => {
            setCloseHides(next)
            setPreferredCloseHides(next)
          }))
        }} />
      <Button id="service-tray-toggle" label={trayEnabled ? 'Disable tray' : 'Enable tray'} theme={props.theme}
        onClick={() => {
          const next = !trayEnabled
          run('Tray', (signal) => props.services.setTrayEnabled(next, signal).then(() => {
            setTrayEnabled(next)
            setCloseHides(next ? preferredCloseHides : false)
          }))
        }} />
      <Button id="service-open-companion" label="Open companion window" theme={props.theme}
        onClick={() => run('Companion', (signal) => props.services.call('windows', 'openCompanion', {
          title: windowTitle, width: Number(windowWidth), height: Number(windowHeight),
          decorations, transparent, backdrop, backdropScope,
        }, signal))} />
      <Button id="service-send-companion" label="Send to companion" theme={props.theme}
        onClick={() => run('Send message', (signal) => props.services.call('windows', 'sendMessage', { message }, signal))} />
      <Button id="service-cancel" label="Cancel request" theme={props.theme}
        onClick={() => active.current?.abort()} />
    </row>
    <InputField id="service-companion-message" label="Message to companion" theme={props.theme}
      value={message} onChange={setMessage} />
    <text text="Window options" color={props.theme.foreground} font_size={16} />
    <InputField id="service-window-title" label="Window title" theme={props.theme}
      value={windowTitle} onChange={setWindowTitle} />
    <row wrap={true} gap={8}>
      <InputField id="service-window-width" label="Width" theme={props.theme}
        value={windowWidth} onChange={setWindowWidth} />
      <InputField id="service-window-height" label="Height" theme={props.theme}
        value={windowHeight} onChange={setWindowHeight} />
    </row>
    <row wrap={true} gap={8}>
      <Button id="service-window-target" label={`Target: ${targetWindow}`} theme={props.theme}
        onClick={() => setTargetWindow(targetWindow === 'main' ? 'companion' : 'main')} />
      <Button id="service-window-decorations" label={`Borders: ${decorations ? 'on' : 'off'}`} theme={props.theme}
        onClick={() => setDecorations(!decorations)} />
      <Button id="service-window-transparent" label={`Transparent: ${transparent ? 'on' : 'off'}`} theme={props.theme}
        onClick={() => setTransparent(!transparent)} />
      <Button id="service-window-backdrop" label={`Desktop blur: ${backdrop ? 'on' : 'off'}`} theme={props.theme}
        onClick={() => setBackdrop(!backdrop)} />
      <Button id="service-window-backdrop-scope" label={`Blur area: ${backdropScope}`} theme={props.theme}
        onClick={() => setBackdropScope(backdropScope === 'window' ? 'panel' : 'window')} />
      <Button id="service-window-info" label="Read window info" theme={props.theme}
        onClick={() => run('Window info', (signal) => props.services.getWindowInfo(targetWindow, signal))} />
      <Button id="service-window-set-title" label="Apply title" theme={props.theme}
        onClick={() => run('Window title', (signal) => props.services.setWindowTitle(targetWindow, windowTitle, signal))} />
      <Button id="service-window-set-size" label="Apply size" theme={props.theme}
        onClick={() => run('Window size', (signal) => props.services.setWindowSize(targetWindow, Number(windowWidth), Number(windowHeight), signal))} />
      <Button id="service-window-set-decorations" label="Apply borders" theme={props.theme}
        onClick={() => run('Window borders', (signal) => props.services.setWindowDecorations(targetWindow, decorations, signal))} />
    </row>
    <text width="fill" text="Desktop blur can cover the whole companion or just a panel. If unavailable, its solid fallback remains readable. These options apply when opening the window." color={props.theme.muted} font_size={13} />
    <text width="fill" text={status} color={props.theme.muted} font_size={13} />
    <text width="fill" text="Click the companion window to send its reply back here." color={props.theme.muted} font_size={13} />
    <text width="fill" text="The tray icon stays visible when the window closes. Its menu can reopen or quit the app." color={props.theme.muted} font_size={13} />
    <text width="fill" text="Cmd/Ctrl+H wakes the app after Set shortcut. Shortcut registration may require desktop permission." color={props.theme.muted} font_size={13} />
  </column>
}
