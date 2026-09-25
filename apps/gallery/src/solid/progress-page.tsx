import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, Progress, type Palette } from '@argui/widgets/solid'

/** Demonstrates determinate updates and the native indeterminate progress state. */
export function ProgressPage(props: { theme: Palette }): JSX.Element {
  const [progress, setProgress] = createSignal(13)
  const advance = () => setProgress((value) => value >= 100 ? 13 : Math.min(100, value + 27))
  return <column width="fill" gap={16}>
    <text text={`Download progress: ${progress()}%`} color={props.theme.foreground} font_size={14} />
    <Progress theme={props.theme} label="Download" value={progress()} height={10} />
    <Button id="progress-advance" label="Advance progress" theme={props.theme}
      kind="secondary" onClick={advance} />
    <text text="Background task (indeterminate)" color={props.theme.muted} font_size={13} />
    <Progress theme={props.theme} label="Background task" value={null} width={320} />
  </column>
}
