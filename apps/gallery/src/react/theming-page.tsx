/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import { Button, InputEditController, type Palette } from '@argui/widgets/react'
import { numberVariable, parseThemeVariables, stringVariable, themeJson, themePresets,
  validateExampleTheme, type ThemeVariables } from '../theme-variables'

/** Exercises the same app-owned JSON theme through the React renderer. */
export function ReactThemingPage(props: { theme: Palette }): ReactElement {
  const [variables, setVariables] = useState<ThemeVariables>({ ...themePresets.Aurora })
  const [draft, setDraft] = useState(themeJson(themePresets.Aurora))
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(draft)
  const [status, setStatus] = useState('Edit the JSON or choose a preset.')
  const useVariables = (next: ThemeVariables, message: string) => {
    setVariables(next)
    setDraft(themeJson(next))
    setStatus(message)
  }
  const loadJson = () => {
    try {
      const next = parseThemeVariables(draft)
      validateExampleTheme(next)
      useVariables(next, 'JSON theme applied at runtime.')
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error))
    }
  }
  const changeNumber = (name: string, delta: number) => {
    const next = { ...variables, [name]: numberVariable(variables, name) + delta }
    validateExampleTheme(next)
    useVariables(next, `${name} changed at runtime.`)
  }
  return <column width="fill" gap={14}>
    <text width="fill" text="These variable names belong to this app. Use any name in any TSX property of the matching type."
      color={props.theme.muted} font_size={13} />
    <row wrap={true} gap={8}>
      {Object.entries(themePresets).map(([name, preset]) =>
        <Button key={name} id={`theme-preset-${name}`} label={name} theme={props.theme}
          onClick={() => useVariables({ ...preset }, `${name} theme applied.`)} />)}
      <Button id="theme-radius" label={`Radius ${numberVariable(variables, 'radius')}`} theme={props.theme}
        onClick={() => changeNumber('radius', numberVariable(variables, 'radius') >= 40 ? -40 : 8)} />
      <Button id="theme-padding" label={`Padding ${numberVariable(variables, 'padding')}`} theme={props.theme}
        onClick={() => changeNumber('padding', numberVariable(variables, 'padding') >= 40 ? -32 : 8)} />
      <Button id="theme-blur" label={`Blur ${numberVariable(variables, 'blur')}px`} theme={props.theme}
        onClick={() => changeNumber('blur', numberVariable(variables, 'blur') >= 40 ? -40 : 10)} />
    </row>
    <text width="fill" text="Backdrop blur softens the colored shapes behind the translucent panel. Paper starts at 0px; each click adds 10px."
      color={props.theme.muted} font_size={13} />
    <rectangle width="fill" height={340} clip={true} radius={numberVariable(variables, 'radius')}
      background={stringVariable(variables, 'canvas')}>
      <rectangle x={-30} y={65} width={175} height={175} radius={88} background="#f97316" />
      <rectangle x={145} y={-48} width={205} height={205} radius={103} background="#2563eb" />
      <rectangle x={330} y={116} width={160} height={160} radius={80} background="#e11d48" />
      <rectangle width="fill" height="fill" radius={numberVariable(variables, 'radius')}
        background={stringVariable(variables, 'panel')}
        backdrop_filter={`blur(${numberVariable(variables, 'blur')}px)`}>
        <column width="fill" height="fill" gap={14} padding={numberVariable(variables, 'padding')}>
        <text width="fill" text="A theme changed while the app is running" color={stringVariable(variables, 'ink')}
          font_size={numberVariable(variables, 'fontSize')} />
        <text width="fill" text="Color, type, spacing, corners, shadow and blur come from the same variable map."
          color={stringVariable(variables, 'muted')} font_size={14} />
        <text text="Themed text surface" padding={numberVariable(variables, 'padding')}
          radius={numberVariable(variables, 'radius')} background={stringVariable(variables, 'accent')}
          color={stringVariable(variables, 'canvas')} font_size={16} />
        <rectangle width={90} height={36} background={stringVariable(variables, 'accent')}
          radius={numberVariable(variables, 'radius')} shadow_blur={numberVariable(variables, 'shadowBlur')}
          shadow_color={stringVariable(variables, 'shadowColor')} />
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
        onClick={() => { setDraft(themeJson(variables)); setStatus('Current theme serialized as JSON.') }} />
    </row>
    <text width="fill" text={status} color={props.theme.muted} font_size={13} />
  </column>
}
