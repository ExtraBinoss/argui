/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import type { BrushValue } from '@argui/host'
import { Button, type WidgetTheme } from '@argui/widgets/react'

const edge: BrushValue = { kind: 'linear', angle: 0, space: 'oklab', stops: [
  { offset: 0, color: '#35383d' }, { offset: 0.36, color: '#35383d' },
  { offset: 0.53, color: '#78c9ce' }, { offset: 0.68, color: '#eec49d' },
  { offset: 0.82, color: '#c79bd9' }, { offset: 1, color: '#35383d' },
] }
const aqua: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.94 }, radius: { x: 0.5, y: 0.85 }, stops: [
  { offset: 0, color: '#5de5e2df' }, { offset: 0.26, color: '#5de5e277' }, { offset: 1, color: '#5de5e200' },
] }
const mint: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.94 }, radius: { x: 0.5, y: 0.85 }, stops: [
  { offset: 0, color: '#a0e7baca' }, { offset: 0.28, color: '#a0e7ba68' }, { offset: 1, color: '#a0e7ba00' },
] }
const amber: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.94 }, radius: { x: 0.5, y: 0.85 }, stops: [
  { offset: 0, color: '#ffd098d9' }, { offset: 0.3, color: '#ffd09872' }, { offset: 1, color: '#ffd09800' },
] }
const coral: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.94 }, radius: { x: 0.5, y: 0.85 }, stops: [
  { offset: 0, color: '#ff9f9dcf' }, { offset: 0.28, color: '#ff9f9d6b' }, { offset: 1, color: '#ff9f9d00' },
] }
const violet: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.94 }, radius: { x: 0.5, y: 0.85 }, stops: [
  { offset: 0, color: '#bf9ce8da' }, { offset: 0.28, color: '#bf9ce871' }, { offset: 1, color: '#bf9ce800' },
] }
const aquaHalo: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.5 }, radius: { x: 0.5, y: 0.5 }, stops: [
  { offset: 0, color: '#5de5e28c' }, { offset: 0.45, color: '#5de5e232' }, { offset: 1, color: '#5de5e200' },
] }
const violetHalo: BrushValue = { kind: 'radial', center: { x: 0.5, y: 0.5 }, radius: { x: 0.5, y: 0.5 }, stops: [
  { offset: 0, color: '#bf9ce885' }, { offset: 0.45, color: '#bf9ce82e' }, { offset: 1, color: '#bf9ce800' },
] }
const beam: BrushValue = { kind: 'linear', angle: 0, space: 'oklab', stops: [
  { offset: 0, color: '#68dce500' }, { offset: 0.22, color: '#68dce5af' },
  { offset: 0.52, color: '#fbcd9cf2' }, { offset: 0.77, color: '#cba3eead' },
  { offset: 1, color: '#cba3ee00' },
] }
const spinnerDots = Array.from({ length: 12 }, (_, index) => {
  const angle = index * Math.PI / 6
  return { x: 23 + Math.sin(angle) * 20, y: 23 - Math.cos(angle) * 20,
    color: `rgba(245,247,252,${(0.18 + index * 0.071).toFixed(2)})` }
})

/** Presents five expressive surfaces using retained native motion and GPU gradients. */
export function ExpressivePage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [playing, setPlaying] = useState(true)
  const [glow, setGlow] = useState(false)
  const [voice, setVoice] = useState(0)

  return <column id="expressive-examples" width="100%" gap={20}>
    <column width="100%" gap={6}>
      <text color={theme.text} fontSize={27} weight={600}>Expressive UI</text>
      <text color={theme.textMuted}>Quiet surfaces, considered motion. Click the glow and voice controls to see their states.</text>
    </column>
    <Button id="expressive-playback" variant="outline" onClick={() => setPlaying((value) => !value)}>
      {playing ? 'Pause motion' : 'Resume motion'}
    </Button>

    <rectangle width="100%" maxWidth={790} padding={18} radii={20} background={theme.card} border={{ width: 1, color: theme.border }}>
      <column width="100%" gap={12}>
        <row gap={10} alignItems="center">
          <text color={theme.textMuted} fontSize={12} weight={600}>01</text>
          <text color={theme.text} fontSize={17} weight={600}>Aurora action</text>
        </row>
        <text color={theme.textMuted} fontSize={12}>A traveling edge and a soft bloom answer the click.</text>
        <rectangle width="100%" height={154} radii={17} background="#0e0f12" clip={true}>
          <row width="100%" height="100%" alignItems="center" justifyContent="center">
            <rectangle width={460} height={108} background="transparent">
              <rectangle position="absolute" inset={{ left: 30, top: 16 }} width={380} height={115} background={aquaHalo}
                opacity={glow ? 0.72 : 0} transitionMs={380} loopMs={2400} loopTranslateX={22} loopScale={1.1} loopPlaying={playing && glow} />
              <rectangle position="absolute" inset={{ left: 116, top: 16 }} width={290} height={115} background={violetHalo}
                opacity={glow ? 0.68 : 0} transitionMs={380} loopMs={3100} loopTranslateX={-28} loopScale={1.15} loopPlaying={playing && glow} />
              <row width="100%" height="100%" alignItems="center" justifyContent="center">
                <focusScope id="expressive-glow-button" role="button" accessibleName={glow ? 'Hide the glow' : 'Reveal the glow'}
                  pressedState={glow} keyboardActivation="enterOrSpace" mouseCursor="pointer" pressBounceScale={0.98}
                  onClick={() => setGlow((value) => !value)} width={440}>
                  <rectangle width="100%" height={66} radii={33} background="#303239" clip={true} focusBorderColor="#a8d8df">
                    <rectangle id="expressive-click-gradient" position="absolute" inset={{ left: -60, top: 0 }}
                      width={560} height={66} background={edge} opacity={glow ? 0.95 : 0} transitionMs={320}
                      loopMs={3400} loopTranslateX={100} loopPlaying={playing && glow} />
                    <rectangle position="absolute" inset={{ left: 2, top: 2 }} width={436} height={62} radii={31}
                      background="#202126" hoverBackground="#282a30" transitionMs={170} />
                    <row width="100%" height="100%" padding={{ left: 23, right: 14 }} alignItems="center" justifyContent="spaceBetween">
                      <row gap={11} alignItems="center">
                        <text color="#d6dadf" fontSize={18}>✦</text>
                        <text color="#f4f5f7" fontSize={15} weight={500}>{glow ? 'The glow is alive' : 'Reveal the glow'}</text>
                      </row>
                      <rectangle width={34} height={34} radii={17} background="#34363e">
                        <row width="100%" height="100%" alignItems="center" justifyContent="center">
                          <text color="#f4f5f7" fontSize={17}>↑</text>
                        </row>
                      </rectangle>
                    </row>
                  </rectangle>
                </focusScope>
              </row>
            </rectangle>
          </row>
        </rectangle>
      </column>
    </rectangle>

    <rectangle width="100%" maxWidth={790} padding={18} radii={20} background={theme.card} border={{ width: 1, color: theme.border }}>
      <column width="100%" gap={12}>
        <row gap={10} alignItems="center">
          <text color={theme.textMuted} fontSize={12} weight={600}>02</text>
          <text color={theme.text} fontSize={17} weight={600}>Voice, then thought</text>
        </row>
        <text color={theme.textMuted} fontSize={12}>The bloom breathes while listening; processing gathers it into a moving beam.</text>
        <rectangle width="100%" height={150} radii={17} background="#0e0f12" clip={true}>
          <row width="100%" height="100%" alignItems="center" justifyContent="center">
            <focusScope id="expressive-record-button" role="button" accessibleName={voice === 0 ? 'Start voice demo' : voice === 1 ? 'Process voice demo' : 'Reset voice demo'}
              keyboardActivation="enterOrSpace" mouseCursor="pointer" pressBounceScale={0.98}
              onClick={() => setVoice((value) => (value + 1) % 3)} width={380}>
              <rectangle width="100%" height={78} radii={39} background="#242528" hoverBackground="#2a2b30"
                border={{ width: 1, color: '#393a3e' }} focusBorderColor="#a8d8df" clip={true}>
                <rectangle id="expressive-voice-cyan" position="absolute" inset={{ left: -18, top: -4 }} width={150} height={100}
                  background={aqua} opacity={voice === 1 ? 0.76 : 0} transitionMs={360}
                  loopMs={1800} loopTranslateX={18} loopScale={1.15} loopPlaying={playing && voice === 1} />
                <rectangle position="absolute" inset={{ left: 45, top: -2 }} width={160} height={98}
                  background={mint} opacity={voice === 1 ? 0.65 : 0} transitionMs={360}
                  loopMs={2200} loopTranslateX={-15} loopScale={1.11} loopPlaying={playing && voice === 1} />
                <rectangle position="absolute" inset={{ left: 116, top: -5 }} width={172} height={102}
                  background={amber} opacity={voice === 1 ? 0.67 : 0} transitionMs={360}
                  loopMs={2700} loopTranslateX={18} loopScale={1.13} loopPlaying={playing && voice === 1} />
                <rectangle position="absolute" inset={{ left: 212, top: -4 }} width={154} height={100}
                  background={coral} opacity={voice === 1 ? 0.72 : 0} transitionMs={360}
                  loopMs={2000} loopTranslateX={-19} loopScale={1.16} loopPlaying={playing && voice === 1} />
                <rectangle position="absolute" inset={{ left: 276, top: -4 }} width={156} height={100}
                  background={violet} opacity={voice === 1 ? 0.7 : 0} transitionMs={360}
                  loopMs={2400} loopTranslateX={-18} loopScale={1.12} loopPlaying={playing && voice === 1} />
                <rectangle position="absolute" inset={{ left: -90, top: 21 }} width={270} height={85}
                  background={beam} opacity={voice === 2 ? 0.82 : 0} transitionMs={340}
                  loopMs={1150} loopTranslateX={290} loopPlaying={playing && voice === 2} />
                <row width="100%" height="100%" padding={{ left: 24, right: 12 }} alignItems="center" justifyContent="spaceBetween">
                  <row gap={12} alignItems="center">
                    <text color="#c7cbd2" fontSize={20}>◉</text>
                    <text color="#f5f5f6" fontSize={18} weight={500}>{voice === 0 ? 'Tap to speak' : voice === 1 ? 'Listening…' : 'Processing…'}</text>
                  </row>
                  <rectangle width={48} height={48} radii={24} background="#38393c" border={{ width: 1, color: '#4a4b4f' }}>
                    <row width="100%" height="100%" alignItems="center" justifyContent="center">
                      <text color="#f4f4f5" fontSize={18}>{voice === 1 ? '■' : voice === 2 ? '↻' : '●'}</text>
                    </row>
                  </rectangle>
                </row>
              </rectangle>
            </focusScope>
          </row>
        </rectangle>
      </column>
    </rectangle>

    <rectangle width="100%" maxWidth={790} padding={18} radii={20} background={theme.card} border={{ width: 1, color: theme.border }}>
      <column width="100%" gap={12}>
        <row gap={10} alignItems="center">
          <text color={theme.textMuted} fontSize={12} weight={600}>03</text>
          <text color={theme.text} fontSize={17} weight={600}>Dotted thought</text>
        </row>
        <text color={theme.textMuted} fontSize={12}>A crisp twelve-dot throbber and a softly breathing status line.</text>
        <rectangle width="100%" height={132} radii={17} background="#0e0f12">
          <row width="100%" height="100%" alignItems="center" justifyContent="center">
            <rectangle width={370} height={68} radii={34} background="#202124" border={{ width: 1, color: '#303136' }}>
              <row width="100%" height="100%" gap={17} padding={{ left: 23, right: 18 }} alignItems="center">
                <rectangle id="expressive-dotted-spinner" width={52} height={52} background="transparent"
                  rotationLoopMs={1200} loopPlaying={playing}>
                  {spinnerDots.map((dot, index) => <rectangle key={index} position="absolute" inset={{ left: dot.x, top: dot.y }}
                    width={4.5} height={4.5} radii={2.25} background={dot.color} />)}
                </rectangle>
                <rectangle background="transparent" loopMs={1950} loopOpacity={0.72} loopPlaying={playing}>
                  <text color="#f5f5f6" fontSize={20} weight={500}>Generating video…</text>
                </rectangle>
              </row>
            </rectangle>
          </row>
        </rectangle>
      </column>
    </rectangle>

    <rectangle width="100%" maxWidth={790} padding={18} radii={20} background={theme.card} border={{ width: 1, color: theme.border }}>
      <column width="100%" gap={12}>
        <row gap={10} alignItems="center">
          <text color={theme.textMuted} fontSize={12} weight={600}>04</text>
          <text color={theme.text} fontSize={17} weight={600}>Living status</text>
        </row>
        <text color={theme.textMuted} fontSize={12}>Three restrained pulses make a compact, readable waiting state.</text>
        <rectangle width="100%" height={132} radii={17} background="#0e0f12">
          <row width="100%" height="100%" alignItems="center" justifyContent="center">
            <rectangle width={326} height={66} radii={33} background="#202124" border={{ width: 1, color: '#303136' }}>
              <row width="100%" height="100%" padding={{ left: 25, right: 21 }} alignItems="center" gap={18}>
                <row gap={6} alignItems="center">
                  <rectangle id="expressive-breathing-halo" width={9} height={9} radii={5} background="#9dded3"
                    loopMs={600} loopScale={1.42} loopOpacity={0.36} loopPlaying={playing} />
                  <rectangle width={9} height={9} radii={5} background="#ecc3a9"
                    loopMs={780} loopScale={1.42} loopOpacity={0.36} loopPlaying={playing} />
                  <rectangle width={9} height={9} radii={5} background="#bea5df"
                    loopMs={960} loopScale={1.42} loopOpacity={0.36} loopPlaying={playing} />
                </row>
                <text color="#f5f5f6" fontSize={18} weight={500}>Planning next steps</text>
              </row>
            </rectangle>
          </row>
        </rectangle>
      </column>
    </rectangle>

    <rectangle width="100%" maxWidth={790} padding={18} radii={20} background={theme.card} border={{ width: 1, color: theme.border }}>
      <column width="100%" gap={12}>
        <row gap={10} alignItems="center">
          <text color={theme.textMuted} fontSize={12} weight={600}>05</text>
          <text color={theme.text} fontSize={17} weight={600}>Progress beam</text>
        </row>
        <text color={theme.textMuted} fontSize={12}>A luminous gradient travels through a fixed track, leaving the layout untouched.</text>
        <rectangle width="100%" height={134} radii={17} background="#0e0f12">
          <row width="100%" height="100%" alignItems="center" justifyContent="center">
            <rectangle width={398} height={76} radii={19} background="#1d1f23" border={{ width: 1, color: '#303239' }}>
              <column width="100%" padding={{ left: 22, right: 22, top: 14, bottom: 13 }} gap={12}>
                <row width="100%" alignItems="center" justifyContent="spaceBetween">
                  <text color="#f5f5f6" fontSize={15} weight={500}>Building your preview</text>
                  <text color="#9da1ad" fontSize={12}>IN PROGRESS</text>
                </row>
                <rectangle width="100%" height={8} radii={4} background="#303238" clip={true}>
                  <rectangle id="expressive-sweep" position="absolute" inset={{ left: -110, top: 0 }}
                    width={190} height={8} radii={4} background={beam} loopMs={1450} loopTranslateX={470}
                    loopPlaying={playing} />
                </rectangle>
              </column>
            </rectangle>
          </row>
        </rectangle>
      </column>
    </rectangle>
  </column>
}
