/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Button, type WidgetTheme } from '@argui/widgets/react'

/** Demonstrates native loops with explanatory labels inside the moving surfaces. */
export function AnimationPage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [playing, setPlaying] = useState(true)

  return <column id="animation-examples" width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Animation</text>
    <text color={theme.textMuted}>These loops run in the native renderer. Pause them to inspect each starting state; no JavaScript timer drives a frame.</text>
    <Button id="animation-playback" variant="secondary" onClick={() => setPlaying((value) => !value)}>
      {playing ? 'Pause animations' : 'Resume animations'}
    </Button>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Travel · loopTranslateX</text>
      <text color={theme.textMuted}>The caption rides inside its rectangle as the native transform travels 120 px and returns. Transform changes paint position without changing the row's layout.</text>
      <rectangle id="animation-travel-track" width="100%" height={64} padding={8} background={theme.surfaceHover} radii={8}>
        <rectangle id="animation-travel" width={150} height={48} padding={8} background={theme.primary} radii={6}
          loopMs={1300} loopTranslateX={120} loopPlaying={playing}>
          <text color={theme.primaryForeground}>Traveling text</text>
        </rectangle>
      </rectangle>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Pulse · loopScale</text>
      <text color={theme.textMuted}>The whole rectangle, including its text, grows to 1.18× and returns. Its reserved layout width stays at 180 px.</text>
      <rectangle id="animation-scale-track" width="100%" height={76} padding={12} background={theme.surfaceHover} radii={8}>
        <rectangle id="animation-scale" width={180} height={48} padding={8} background={theme.primary} radii={6}
          loopMs={900} loopScale={1.18} loopPlaying={playing}>
          <text color={theme.primaryForeground}>Pulse with text</text>
        </rectangle>
      </rectangle>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Fade and color · loopOpacity / loopBackground</text>
      <text color={theme.textMuted}>The native paint loop fades the complete rectangle to 45% and blends its background color. The text remains part of the same animated surface.</text>
      <rectangle id="animation-color-track" width="100%" height={64} padding={8} background={theme.surfaceHover} radii={8}>
        <rectangle id="animation-color" width={230} height={48} padding={8} background={theme.primary} radii={6}
          loopMs={1100} loopOpacity={0.45} loopBackground={theme.danger} loopPlaying={playing}>
          <text color={theme.primaryForeground}>Fade and change color</text>
        </rectangle>
      </rectangle>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Layout motion · loopWidth</text>
      <text color={theme.textMuted}>Unlike a transform, width animates the rectangle's layout from 180 to 280 px. Watch the caption stay inside its changing bounds.</text>
      <rectangle id="animation-width-track" width="100%" height={64} padding={8} background={theme.surfaceHover} radii={8}>
        <rectangle id="animation-width" width={180} height={48} padding={8} background={theme.primary} radii={6}
          loopMs={1400} loopWidth={280} loopPlaying={playing}>
          <text color={theme.primaryForeground}>Width 180 → 280</text>
        </rectangle>
      </rectangle>
    </column>
  </column>
}
