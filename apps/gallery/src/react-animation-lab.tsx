/** @jsxImportSource @argui/react */
import { useState, type ReactElement, type ReactNode } from 'react'
import { ReactButton } from './react-controls'
import type { Palette } from './theme'

interface MotionCardProps {
  title: string
  detail: string
  theme: Palette
  children: ReactNode
}

/** Frames one primitive animation against the gallery's current palette. */
function MotionCard(props: MotionCardProps): ReactElement {
  return (
    <column width={240} min_width={220} grow={1} gap={9} padding={14}
      background={props.theme.surface} border_color={props.theme.border} radius={12}>
      <text text={props.title} color={props.theme.foreground} font_size={15} weight={650} />
      <text text={props.detail} color={props.theme.muted} font_size={12} />
      <rectangle width="fill" height={104} radius={9} clip={true}
        background={props.theme.surfaceRaised}>{props.children}</rectangle>
    </column>
  )
}

interface MotionSectionProps {
  title: string
  detail: string
  theme: Palette
  children: ReactNode
}

/** Groups related examples and lets the cards wrap on narrow screens. */
function MotionSection(props: MotionSectionProps): ReactElement {
  return (
    <column width="fill" gap={9}>
      <text text={props.title} color={props.theme.foreground} font_size={19} weight={700} />
      <text text={props.detail} color={props.theme.muted} font_size={13} />
      <row width="fill" wrap={true} gap={12}>{props.children}</row>
    </column>
  )
}

/** Restores the gallery's implicit, timeline, composition and physics examples. */
export function ReactAnimationLab(props: { theme: Palette }): ReactElement {
  const [expanded, setExpanded] = useState(false)
  const [playing, setPlaying] = useState(true)
  const [step, setStep] = useState(0)
  const active = () => expanded
  const dot = (index: number) => step >= index ? props.theme.accent : props.theme.border

  return (
    <column width="fill" gap={20}>
      <text text="Native loops run on the UI clock; buttons retarget discrete examples."
        color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={10}>
        <ReactButton id="motion-target" label={expanded ? 'Use compact target' : 'Change target'}
          theme={props.theme} kind="secondary" onClick={() => setExpanded((value) => !value)} />
        <ReactButton id="motion-play" label={playing ? 'Pause loops' : 'Resume loops'}
          theme={props.theme} kind="secondary" onClick={() => setPlaying((value) => !value)} />
        <ReactButton id="motion-step" label="Next keyframe" theme={props.theme} kind="quiet"
          onClick={() => setStep((value) => (value + 1) % 5)} />
      </row>
      <MotionSection title="Timeline and keyframes" detail="Declarative loops advance on the native frame clock." theme={props.theme}>
        <MotionCard title="Travel" detail="Native transform translation" theme={props.theme}>
          <rectangle x={16} y={28} width={46} height={46} radius={13}
            background={props.theme.accent} loop_ms={1300} loop_translate_x={164} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Keyframe spin" detail="Continuous native rotation" theme={props.theme}>
          <rectangle x={92} y={24} width={56} height={56} radius={12}
            background={props.theme.accent} rotation_loop_ms={2400} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Pulse" detail="Scale and opacity share a phase" theme={props.theme}>
          <rectangle x={82} y={22} width={52} height={52}
            radius={40} loop_ms={800} loop_scale={1.5} loop_opacity={0.45}
            loop_playing={playing} background={props.theme.accent} />
        </MotionCard>
        <MotionCard title="Five steps" detail="Native pulses; Next keyframe changes colors" theme={props.theme}>
          {[0, 1, 2, 3, 4].map((index) => (
            <rectangle key={`step-${index}`} nativeKey={`step-${index}`} x={24 + index * 36} y={37} width={28} height={28}
              radius={7} background={dot(index)} loop_ms={650 + index * 140}
              loop_opacity={0.35} loop_playing={playing} />
          ))}
        </MotionCard>
      </MotionSection>

      <MotionSection title="Implicit transitions" detail="Native loops stay active; Change target retargets layout and paint properties." theme={props.theme}>
        <MotionCard title="Opacity" detail="Native fade loop" theme={props.theme}>
          <rectangle x={80} y={22} width={72} height={60} radius={15} background={props.theme.accent}
            loop_ms={850} loop_opacity={0.25} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Size" detail="Native scale pulse; width retargets on click" theme={props.theme}>
          <rectangle x={18} y={26} width={active() ? 150 : 74} height={52} radius={14}
            background={props.theme.accent} transition_ms={520} loop_ms={1000}
            loop_scale={1.25} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Rounded corners" detail="Native radius loop; Change target reverses it" theme={props.theme}>
          <rectangle x={82} y={20} width={68} height={68} radius={active() ? 34 : 7}
            background={props.theme.accent} transition_ms={380} loop_ms={1100}
            loop_radius={active() ? 7 : 34} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Rotation" detail="Continuous compositor transform" theme={props.theme}>
          <rectangle x={89} y={24} width={58} height={58} radius={11}
            background={props.theme.accent} rotation_loop_ms={1800} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Layout gap" detail="Children shift; gap retargets on click" theme={props.theme}>
          <row x={28} y={37} gap={active() ? 24 : 5} transition_ms={480}>
            {[0, 1, 2].map((index) => <rectangle key={`gap-${index}`} nativeKey={`gap-${index}`}
              width={28} height={28} radius={6}
              background={index === 1 ? props.theme.border : props.theme.accent}
              loop_ms={index === 1 ? undefined : 1100}
              loop_translate_x={index === 0 ? -9 : index === 2 ? 9 : undefined}
              loop_playing={playing} />)}
          </row>
        </MotionCard>
        <MotionCard title="Text" detail="Native text fade; color retargets on click" theme={props.theme}>
          <text x={43} y={38} text="Argui" color={active() ? '#ef4444' : props.theme.accent}
            transition_ms={500} font_size={24} weight={700}
            loop_ms={950} loop_opacity={0.2} loop_playing={playing} />
        </MotionCard>
      </MotionSection>

      <MotionSection title="Composition" detail="Several primitives respond to native loops or button state." theme={props.theme}>
        <MotionCard title="Opposing tracks" detail="Two native translation loops" theme={props.theme}>
          <rectangle x={12} y={20} width={42} height={26} radius={9}
            background={props.theme.accent} loop_ms={1300} loop_translate_x={145} loop_playing={playing} />
          <rectangle x={157} y={58} width={42} height={26} radius={9}
            background={props.theme.border} loop_ms={1300} loop_translate_x={-145} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Color transition" detail="Change target to retarget color" theme={props.theme}>
          <rectangle x={76} y={18} width={78} height={68} radius={18}
            background={active() ? props.theme.border : props.theme.accent} transition_ms={500} />
        </MotionCard>
        <MotionCard title="Stagger" detail="Native offsets; Next keyframe retargets rows" theme={props.theme}>
          {[0, 1, 2, 3, 4].map((index) => (
            <rectangle key={`stagger-${index}`} nativeKey={`stagger-${index}`} x={31 + index * 34}
              y={step >= index ? 28 : 48}
              width={24} height={24} radius={12} background={props.theme.accent} transition_ms={180}
              loop_ms={700 + index * 150} loop_translate_y={-14} loop_playing={playing} />
          ))}
        </MotionCard>
        <MotionCard title="Retargeting" detail="Native pulse; click to move target" theme={props.theme}>
          <rectangle x={active() ? 158 : 18} y={28} width={48} height={48} radius={12}
            background={props.theme.accent} transition_ms={700} loop_ms={950}
            loop_scale={1.2} loop_playing={playing} />
        </MotionCard>
      </MotionSection>

      <MotionSection title="Physics and timing" detail="Spring retargeting and discrete keyframes." theme={props.theme}>
        <MotionCard title="Spring retarget" detail="Native fade; click for spring rotation" theme={props.theme}>
          <rectangle x={87} y={24} width={58} height={58} radius={12}
            rotation={active() ? 130 : 0} background={props.theme.accent} transition_spring={true}
            loop_ms={850} loop_opacity={0.55} loop_playing={playing} />
        </MotionCard>
        <MotionCard title="Held keyframes" detail="A native timeline pauses at its target" theme={props.theme}>
          <rectangle x={22} y={28} width={47} height={47} radius={12}
            background={props.theme.accent} loop_ms={1800} loop_translate_x={129}
            loop_hold={true} loop_playing={playing} />
        </MotionCard>
      </MotionSection>
    </column>
  )
}
