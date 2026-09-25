/** @jsxImportSource @argui/react */
import { useEffect, useState, type ReactElement } from 'react'
import {
  setNativeDamageTracking, subscribeRendererProfiles,
  type RendererProfileSample,
} from '../native-runtime'
import { Button as ReactButton, type Palette } from '@argui/widgets/react'

/** React adapter for the measured native damage comparison. */
export function ReactDamageControl(props: { theme: Palette; active?: boolean }): ReactElement {
  const [adaptive, setAdaptive] = useState(true)
  const [running, setRunning] = useState(true)
  const [sample, setSample] = useState<RendererProfileSample | null>(null)
  const [status, setStatus] = useState('Waiting for native renderer samples')
  const active = props.active !== false

  useEffect(() => {
    if (!active) return
    setSample(null)
    if (!setNativeDamageTracking(adaptive)) setStatus('Native damage control unavailable')
    const unsubscribe = subscribeRendererProfiles((next) => {
      setSample(next)
      setStatus('Measured on the native renderer')
    })
    if (!unsubscribe) setStatus('Native renderer profiles unavailable')
    return () => {
      unsubscribe?.()
      setNativeDamageTracking(true)
    }
  }, [active, adaptive])

  const choose = (enabled: boolean) => {
    setSample(null)
    setAdaptive(enabled)
  }
  const value = (format: (sample: RendererProfileSample) => string) =>
    sample ? format(sample) : 'Waiting…'

  return (
    <column width="fill" gap={16}>
      <text text="A moving native layer passes beneath real backdrop blur. Compare adaptive damage with full-frame repaint."
        color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={9}>
        <ReactButton id="damage-auto" label="Auto · adaptive" theme={props.theme}
          kind="secondary" selected={adaptive} onClick={() => choose(true)} />
        <ReactButton id="damage-off" label="Off · full frame" theme={props.theme}
          kind="secondary" selected={!adaptive} onClick={() => choose(false)} />
        <ReactButton id="damage-pause" label={running ? 'Pause motion' : 'Resume motion'}
          theme={props.theme} kind="quiet" onClick={() => setRunning((current) => !current)} />
      </row>
      <rectangle width="fill" height={165} radius={13} clip={true}
        background={props.theme.surfaceRaised} border_color={props.theme.border} border_width={1}>
        <rectangle x={22} y={58} width={50} height={50} radius={25}
          background={props.theme.accent} loop_ms={1600} loop_translate_x={200}
          loop_playing={active && running} />
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
      <text text={status} color={props.theme.muted} font_size={12} />
      <text text="GPU uses native timestamp queries when supported; unavailable means the adapter did not report a GPU sample."
        color={props.theme.muted} font_size={12} />
    </column>
  )
}

/** Displays a measured renderer value in the React adapter. */
function Metric(props: { label: string; value: string; theme: Palette }): ReactElement {
  return <column width={190} min_width={170} grow={1} gap={5} padding={13}
    background={props.theme.surface} border_color={props.theme.border} border_left={1} border_right={1} border_top={1} border_bottom={1} radius={10}>
    <text text={props.label} color={props.theme.muted} font_size={11} />
    <text text={props.value} color={props.theme.foreground} font_size={18} weight={700} />
  </column>
}
