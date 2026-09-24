import { createRenderEffect } from 'solid-js'
import { createSignal, onCleanup } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import {
  setNativeDamageTracking, subscribeRendererProfiles,
  type RendererProfileSample,
} from './native-runtime'
import { Button, type Palette } from '@argui/widgets/solid'

/** Compares actual adaptive and full-frame renderer decisions on one workload. */
export function DamageControl(props: { theme: Palette; active?: boolean }): JSX.Element {
  const [adaptive, setAdaptive] = createSignal(true)
  const [running, setRunning] = createSignal(true)
  const [sample, setSample] = createSignal<RendererProfileSample | null>(null)
  const [status, setStatus] = createSignal('Waiting for native renderer samples')

  createRenderEffect(() => {
    if (props.active === false) return
    setSample(null)
    if (!setNativeDamageTracking(adaptive())) setStatus('Native damage control unavailable')
    const unsubscribe = subscribeRendererProfiles((next) => {
      setSample(next)
      setStatus('Measured on the native renderer')
    })
    if (!unsubscribe) setStatus('Native renderer profiles unavailable')
    onCleanup(() => {
      unsubscribe?.()
      setNativeDamageTracking(true)
    })
  })

  const choose = (enabled: boolean) => {
    setSample(null)
    setAdaptive(enabled)
  }
  const value = (format: (sample: RendererProfileSample) => string) =>
    sample() ? format(sample()!) : 'Waiting…'

  return (
    <column width="fill" gap={16}>
      <text text="A moving native layer passes beneath real backdrop blur. Compare adaptive damage with full-frame repaint."
        color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={9}>
        <Button id="damage-auto" label="Auto · adaptive" theme={props.theme}
          kind="secondary" selected={adaptive()} onClick={() => choose(true)} />
        <Button id="damage-off" label="Off · full frame" theme={props.theme}
          kind="secondary" selected={!adaptive()} onClick={() => choose(false)} />
        <Button id="damage-pause" label={running() ? 'Pause motion' : 'Resume motion'}
          theme={props.theme} kind="quiet" onClick={() => setRunning((value) => !value)} />
      </row>
      <rectangle width="fill" height={165} radius={13} clip={true}
        background={props.theme.surfaceRaised} border_color={props.theme.border} border_width={1}>
        <rectangle x={22} y={58} width={50} height={50} radius={25}
          background={props.theme.accent} loop_ms={1600} loop_translate_x={200}
          loop_playing={props.active !== false && running()} />
        <rectangle x={25} y={129} width={270} height={4} radius={2}
          background={props.theme.border} />
        <rectangle x={112} y={30} width={138} height={100} radius={16}
          background="#ffffff55" backdrop_filter="blur(10px)"
          border_color={props.theme.border} border_width={1} />
      </rectangle>
      <row width="fill" wrap={true} gap={10}>
        <Metric label="CPU encode / sampled frame" value={value((next) => `${next.cpuMs.toFixed(2)} ms`)} theme={props.theme} />
        <Metric label="GPU / sampled frame" value={value((next) => next.gpuMs === null ? 'Unavailable' : `${next.gpuMs.toFixed(2)} ms`)} theme={props.theme} />
        <Metric label="Pixels repainted" value={value((next) => next.viewportPixels === 0 ? 'Unavailable' : `${(100 * next.damagedPixels / next.viewportPixels).toFixed(1)}%`)} theme={props.theme} />
        <Metric label="Damage decision" value={value((next) => `${next.damageMode} · ${next.regions} region${next.regions === 1 ? '' : 's'}`)} theme={props.theme} />
      </row>
      <text text={status()} color={props.theme.muted} font_size={12} />
      <text text="GPU uses native timestamp queries when supported; unavailable means the adapter did not report a GPU sample."
        color={props.theme.muted} font_size={12} />
    </column>
  )
}

/** Displays one measured native value without estimating missing samples. */
function Metric(props: { label: string; value: string; theme: Palette }): JSX.Element {
  return <column width={190} min_width={170} grow={1} gap={5} padding={13}
    background={props.theme.surface} border_color={props.theme.border} border_left={1} border_right={1} border_top={1} border_bottom={1} radius={10}>
    <text text={props.label} color={props.theme.muted} font_size={11} />
    <text text={props.value} color={props.theme.foreground} font_size={18} weight={700} />
  </column>
}
