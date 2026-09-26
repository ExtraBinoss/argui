/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Slider, type WidgetTheme } from '@argui/widgets/react'

/** Demonstrates pointer, keyboard, and step-aware slider input. */
export function SliderPage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [volume, setVolume] = useState(40)
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Slider</text>
    <text color={theme.textMuted}>Drag or click the track. Arrow keys change one step; Page keys change ten.</text>
    <text color={theme.text}>{`Volume: ${volume}%`}</text>
    <Slider accessibleName="Volume" value={volume} onValueChange={setVolume} step={5} width={280} />
    <text color={theme.text}>Local value with a restricted range</text>
    <Slider accessibleName="Brightness" min={20} max={80} step={10} defaultValue={50} width={280} />
    <Slider accessibleName="Disabled slider" disabled width={280} />
  </column>
}
