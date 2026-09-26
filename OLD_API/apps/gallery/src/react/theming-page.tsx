/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import type { ThemeRuntime } from '@argui/host'
import { Button, InputEditController, type Palette } from '@argui/widgets/react'
import { parseThemeVariables, previewTokens, themeJson, themePresets,
  type GalleryTokens, type PreviewTokens } from '../theme'

/** Edits the same app-owned typed theme used by the React widget gallery. */
export function ReactThemingPage(props: {
  theme: Palette
  tokens: GalleryTokens
  runtime: ThemeRuntime<GalleryTokens>
}): ReactElement {
  const [draft, setDraft] = useState(themeJson(themePresets.Aurora))
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(draft)
  const [status, setStatus] = useState('Edit the JSON or choose a preset.')
  const apply = (next: PreviewTokens, message: string) => {
    try {
      props.runtime.update({ overrides: next })
      setDraft(themeJson(next))
      setStatus(message)
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error))
    }
  }
  const changeNumber = (name: 'sampleRadius' | 'samplePadding' | 'sampleBlur', delta: number) => {
    const next = { ...previewTokens(props.tokens), [name]: props.tokens[name] + delta }
    apply(next, `${name} changed at runtime.`)
  }
  const changeWidgetMetric = (name: 'controlRadius' | 'controlPadding' | 'controlFontSize' | 'overlayShadowBlur', delta: number) => {
    props.runtime.update({ overrides: { [name]: Math.max(0, props.tokens[name] + delta) } })
    setStatus(`${name} changed across the widget gallery.`)
  }
  const loadJson = () => {
    try { apply(parseThemeVariables(draft), 'JSON theme applied at runtime.') }
    catch (error) { setStatus(error instanceof Error ? error.message : String(error)) }
  }
  const v = props.tokens
  return <column width="fill" gap={14}>
    <text width="fill" text="These tokens belong to this app. Presets and JSON update the same theme as the widget gallery."
      color={props.theme.muted} font_size={13} />
    <row wrap={true} gap={8}>
      {Object.entries(themePresets).map(([name, preset]) =>
        <Button key={name} id={`theme-preset-${name}`} label={name} theme={props.theme}
          onClick={() => apply({ ...preset }, `${name} theme applied.`)} />)}
      <Button id="theme-radius" label={`Radius ${v.sampleRadius}`} theme={props.theme}
        onClick={() => changeNumber('sampleRadius', v.sampleRadius >= 40 ? -40 : 8)} />
      <Button id="theme-padding" label={`Padding ${v.samplePadding}`} theme={props.theme}
        onClick={() => changeNumber('samplePadding', v.samplePadding >= 40 ? -32 : 8)} />
      <Button id="theme-blur" label={`Blur ${v.sampleBlur}px`} theme={props.theme}
        onClick={() => changeNumber('sampleBlur', v.sampleBlur >= 40 ? -40 : 10)} />
    </row>
    <text width="fill" text="Widget tokens update every component page, including text size, spacing, corners and popup shadows."
      color={props.theme.muted} font_size={13} />
    <row wrap={true} gap={8}>
      <Button id="widget-radius" label={`Widget radius ${v.controlRadius}`} theme={props.theme}
        onClick={() => changeWidgetMetric('controlRadius', v.controlRadius >= 20 ? -16 : 4)} />
      <Button id="widget-padding" label={`Widget padding ${v.controlPadding}`} theme={props.theme}
        onClick={() => changeWidgetMetric('controlPadding', v.controlPadding >= 18 ? -12 : 4)} />
      <Button id="widget-font" label={`Widget font ${v.controlFontSize}`} theme={props.theme}
        onClick={() => changeWidgetMetric('controlFontSize', v.controlFontSize >= 22 ? -8 : 2)} />
      <Button id="widget-shadow" label={`Popup shadow ${v.overlayShadowBlur}`} theme={props.theme}
        onClick={() => changeWidgetMetric('overlayShadowBlur', v.overlayShadowBlur >= 26 ? -20 : 6)} />
    </row>
    <text width="fill" text="Backdrop blur softens the colored shapes behind the translucent panel. Paper starts at 0px; each click adds 10px."
      color={props.theme.muted} font_size={13} />
    <rectangle width="fill" height={340} clip={true} radius={v.sampleRadius}
      background={v.sampleCanvas}>
      <rectangle x={-30} y={65} width={175} height={175} radius={88} background="#f97316" />
      <rectangle x={145} y={-48} width={205} height={205} radius={103} background="#2563eb" />
      <rectangle x={330} y={116} width={160} height={160} radius={80} background="#e11d48" />
      <rectangle width="fill" height="fill" radius={v.sampleRadius}
        background={v.samplePanel}
        backdrop_filter={`blur(${v.sampleBlur}px)`}>
        <column width="fill" height="fill" gap={14} padding={v.samplePadding}>
        <text width="fill" text="A theme changed while the app is running" color={v.sampleInk}
          font_size={v.sampleFontSize} />
        <text width="fill" text="Color, type, spacing, corners, shadow and blur come from one typed theme."
          color={v.sampleMuted} font_size={14} />
        <text text="Themed text surface" padding={v.samplePadding}
          radius={v.sampleRadius} background={v.sampleAccent}
          color={v.sampleCanvas} font_size={16} />
        <rectangle width={90} height={36} background={v.sampleAccent}
          radius={v.sampleRadius} shadow_blur={v.sampleShadowBlur}
          shadow_color={v.sampleShadowColor} />
        </column>
      </rectangle>
    </rectangle>
    <text text="Theme JSON (owned by the application)" color={props.theme.foreground} font_size={16} />
    <textInput nativeKey="theme-json" width="fill" height={190} multiline={true} value={draft}
      label="Theme JSON" background={props.theme.surface} text_color={props.theme.foreground}
      caret_color={props.theme.accent} selection_color={props.theme.accent}
      onEdit={(payload) => { edits.current!.apply(payload, draft, setDraft) }} />
    <row wrap={true} gap={8}>
      <Button id="theme-load-json" label="Apply JSON" theme={props.theme} onClick={loadJson} />
      <Button id="theme-export-json" label="Show current JSON" theme={props.theme}
        onClick={() => { setDraft(themeJson(previewTokens(v))); setStatus('Current theme serialized as JSON.') }} />
    </row>
    <text width="fill" text={status} color={props.theme.muted} font_size={13} />
  </column>
}
