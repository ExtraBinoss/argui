/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { VirtualList } from '@argui/react'
import { marqueeMeasurement, type MarqueeTextProps } from '../shared/marquee-text'

export type { MarqueeTextProps } from '../shared/marquee-text'

/** Measures overflow natively, shows an edge shadow, and plays a native marquee on hover. */
export function ReactMarqueeText(props: MarqueeTextProps): ReactElement {
  const [hovered, setHovered] = useState(false)
  const [started, setStarted] = useState(false)
  const [overflow, setOverflow] = useState(0)
  const fontSize = props.fontSize ?? props.theme.controlFontSize
  const height = Math.ceil(fontSize * 1.6)
  const speed = Number.isFinite(props.speed) && (props.speed ?? 0) > 0 ? props.speed! : 45
  const duration = Math.min(60_000, Math.max(300, overflow / speed * 1000))
  const shadowWidth = Number.isFinite(props.shadowWidth) ? Math.max(0, props.shadowWidth!) : 18
  const shadowIntensity = Number.isFinite(props.shadowIntensity)
    ? Math.max(0, Math.min(1, props.shadowIntensity!)) : 1
  return <container nativeKey={props.id} width={props.width} height={height} position="relative">
    <touchArea width="fill" height="fill"
      onPointerEnter={() => { setHovered(true); setStarted(true) }} onPointerLeave={() => setHovered(false)}>
      <VirtualList id={`${props.id}-viewport`} count={1} axis="horizontal" variable={true}
        estimate={props.width} width={props.width} height={height} scrollbarVisible={false}
        shadow={{ color: props.shadowColor, intensity: shadowIntensity,
          width: shadowWidth, left: props.shadowStart ?? false,
          right: (props.shadowEnd ?? true) && props.overflowMarker === undefined
            && (!hovered || props.shadowOnHover === true) }}
        onMeasure={(payload) => {
          const measured = marqueeMeasurement(payload)
          if (measured) setOverflow(Math.max(0, measured.content - measured.viewport))
        }}
        renderItem={() => <rectangle height={height} loop_ms={duration} loop_translate_x={-overflow}
          loop_playing={overflow > 0 && (props.pauseOnLeave === false ? started : hovered)}
          loop_hold={props.holdAtEnd ?? false}>
          <row height="fill" align_items="center">
            <text text={props.text} no_wrap={true} color={props.theme.foreground} font_size={fontSize} />
          </row>
        </rectangle>} />
    </touchArea>
    {overflow > 0 && props.overflowMarker !== undefined && !hovered
      ? <container position="absolute" inset_right={0} height={height}>
        <rectangle height="fill" background={props.shadowColor ?? props.theme.surface}>
          <row height="fill" padding_left={3} align_items="center">
            <text text={props.overflowMarker} no_wrap={true} color={props.theme.foreground} font_size={fontSize} />
          </row>
        </rectangle>
      </container> : null}
  </container>
}
