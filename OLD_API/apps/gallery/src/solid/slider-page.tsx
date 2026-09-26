import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Slider, type Palette } from '@argui/widgets/solid'

/** Demonstrates controlled single-value and two-thumb range sliders. */
export function SliderPage(props: { theme: Palette }): JSX.Element {
  const [volume, setVolume] = createSignal(35)
  const [budget, setBudget] = createSignal([200, 800])
  return <column width="fill" gap={16}>
    <text text={`Volume: ${volume()}`} color={props.theme.foreground} font_size={14} />
    <Slider id="slider-volume" label="Volume" theme={props.theme}
      value={volume()} onValueChange={(values) => setVolume(values[0] ?? 0)} />
    <text text={`Budget: $${budget()[0]} – $${budget()[1]}`} color={props.theme.foreground} font_size={14} />
    <Slider id="slider-budget" label="Budget range" theme={props.theme}
      value={budget()} onValueChange={setBudget} min={0} max={1000} step={10} width={340} />
    <text text="Use the arrow keys, Home or End while a thumb is focused; drag either thumb with a pointer or touch."
      color={props.theme.muted} font_size={13} width="fill" />
  </column>
}
