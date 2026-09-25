/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { registerNativeEffect } from '../native-runtime'
import prismSource from '../prism.wgsl?raw'
import { Button as ReactButton, type Palette } from '@argui/widgets/react'

registerNativeEffect({
  id: 'gallery.examples.prism',
  source: prismSource,
  parameters: [{ name: 'strength', type: 'f32' }, { name: 'frequency', type: 'f32' }],
})

/** React adapter for the same imported WGSL effect and live parameters. */
export function ReactWgslLab(props: { theme: Palette; active?: boolean }): ReactElement {
  const [strength, setStrength] = useState(0.45)
  const [frequency, setFrequency] = useState(18)
  const filter = `effect(gallery.examples.prism strength=${strength.toFixed(2)} frequency=${frequency})`
  return (
    <column width="fill" gap={16}>
      <text text="A fragment effect imported from prism.wgsl?raw and registered before renderer startup."
        color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={9}>
        <ReactButton id="wgsl-strength-down" label="Less tint" theme={props.theme} kind="secondary"
          onClick={() => setStrength((value) => Math.max(0, +(value - 0.1).toFixed(2)))} />
        <ReactButton id="wgsl-strength-up" label="More tint" theme={props.theme} kind="secondary"
          onClick={() => setStrength((value) => Math.min(1, +(value + 0.1).toFixed(2)))} />
        <ReactButton id="wgsl-frequency-down" label="Wider bands" theme={props.theme} kind="quiet"
          onClick={() => setFrequency((value) => Math.max(4, value - 2))} />
        <ReactButton id="wgsl-frequency-up" label="Tighter bands" theme={props.theme} kind="quiet"
          onClick={() => setFrequency((value) => Math.min(32, value + 2))} />
      </row>
      <text text={`Tint ${Math.round(strength * 100)}% · Frequency ${frequency}`}
        color={props.theme.foreground} font_size={14} />
      <rectangle width="fill" height={220} radius={14} clip={true}
        background={props.theme.surfaceRaised}>
        <rectangle x={20} y={34} width={92} height={92} radius={24} background={props.theme.accent}
          loop_ms={1600} loop_translate_x={160} loop_playing={props.active !== false} />
        <rectangle x={150} y={75} width={90} height={90} radius={45} background={props.theme.border} />
        <rectangle x={38} y={24} width={280} height={166} radius={18}
          background="#ffffff55" backdrop_filter={filter} border_color={props.theme.border} border_width={1} />
      </rectangle>
      <text text="This surface is shaded by the imported WGSL. The buttons update GPU parameters without replacing its source."
        color={props.theme.muted} font_size={12} />
    </column>
  )
}
