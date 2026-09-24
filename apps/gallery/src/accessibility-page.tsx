import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { mediaAssets } from './assets.generated'
import type { Palette } from './theme'

/** Exercises authored semantics on native TSX primitives in the Solid gallery. */
export function AccessibilityPage(props: { theme: Palette }): JSX.Element {
  const [activated, setActivated] = createSignal(0)
  const [checked, setChecked] = createSignal(false)
  const [level, setLevel] = createSignal(40)
  const [dialogOpen, setDialogOpen] = createSignal(false)
  const changeLevel = (value: number) => setLevel(Math.max(0, Math.min(100, value)))
  return <column width="fill" gap={16}>
    <text role="heading" level={2} text="Accessibility laboratory"
      color={props.theme.foreground} font_size={21} weight={600} />
    <text text="Try Tab, Enter, Space and a screen reader. Inspect names, roles, states, values and relationships."
      color={props.theme.muted} font_size={14} width="fill" />
    <text key="action-help" text="A Rectangle acting as a button, with a description and keyboard activation."
      color={props.theme.muted} font_size={13} width="fill" />
    <rectangle key="custom-action" role="button" accessible_name="Run accessible action"
      described_by="action-help" focusable keyboard_activation="enter_or_space"
      onClick={() => setActivated((value) => value + 1)}
      width="fill" height={48} radius={8} background={props.theme.accent}>
      <text text="Run accessible action" color={props.theme.accentText} font_size={15} />
    </rectangle>
    <text role="status" live="polite" text={`Action activated ${activated()} times`}
      color={props.theme.foreground} font_size={13} />
    <rectangle role="switch" accessible_name="Enable notifications"
      checked_state={checked() ? 'checked' : 'unchecked'} focusable
      keyboard_activation="enter_or_space" onClick={() => setChecked(!checked())}
      width="fill" height={48} radius={8} background={props.theme.surfaceRaised}>
      <text text={`Notifications: ${checked() ? 'on' : 'off'}`}
        color={props.theme.foreground} font_size={15} />
    </rectangle>
    <text key="level-label" text="Volume" color={props.theme.foreground} font_size={15} />
    <text key="level-help" text="Use the screen reader increase, decrease or set value actions."
      color={props.theme.muted} font_size={13} />
    <rectangle role="slider" labelled_by="level-label" described_by="level-help"
      numeric_value={level()} minimum_value={0} maximum_value={100} value_step={10}
      orientation="horizontal" focusable can_increment can_decrement can_set_value
      onSemanticAction={({ action, value }) => {
        if (action === 'increment') changeLevel(level() + 10)
        if (action === 'decrement') changeLevel(level() - 10)
        if (action === 'set_value' && typeof value === 'number') changeLevel(value)
      }}
      width="fill" height={48} radius={8} background={props.theme.surfaceRaised}>
      <text text={`Volume: ${level()}%`} color={props.theme.foreground} font_size={15} />
    </rectangle>
    <rectangle role="button" accessible_name="Unavailable action" accessible_disabled
      focusable keyboard_activation="enter_or_space" onClick={() => setActivated(999)}
      width="fill" height={42} radius={8} background={props.theme.surfaceRaised}>
      <text text="Unavailable action" color={props.theme.muted} font_size={14} />
    </rectangle>
    <row wrap gap={12}>
      <image source={mediaAssets['illustration/orbit.png']} alt="Colorful orbit around a planet"
        width={180} height={120} fit="contain" />
      <svg source={mediaAssets['tabler/heart.svg']} alt="Heart icon"
        color={props.theme.accent} width={48} height={48} />
      <svg source={mediaAssets['tabler/photo.svg']} alt=""
        color={props.theme.accent} width={48} height={48} />
    </row>
    <rectangle role="button" accessible_name="Open modal example" focusable
      keyboard_activation="enter_or_space" onClick={() => setDialogOpen(true)}
      width="fill" height={42} radius={8} background={props.theme.surfaceRaised}>
      <text text="Open modal example" color={props.theme.foreground} font_size={14} />
    </rectangle>
    {dialogOpen() ? <focusScope role="dialog" accessible_name="Modal example" modal focusable width="fill">
      <column width="fill" gap={12} padding={16} background={props.theme.surfaceRaised}>
      <text text="This dialog exposes a modal role and an accessible name."
        color={props.theme.foreground} font_size={14} />
      <rectangle role="button" accessible_name="Close modal example" focusable
        keyboard_activation="enter_or_space" onClick={() => setDialogOpen(false)}
        width="fill" height={42} background={props.theme.accent}>
        <text text="Close modal" color={props.theme.accentText} font_size={14} />
      </rectangle>
      </column>
    </focusScope> : null}
  </column>
}
