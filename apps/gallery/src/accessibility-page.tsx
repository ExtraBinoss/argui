import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { mediaAssets } from './assets.generated'
import { Button, InputField, Select, type Palette } from '@argui/widgets/solid'

/** Shows accessible components and a custom native slider in Solid. */
export function AccessibilityPage(props: { theme: Palette }): JSX.Element {
  const [count, setCount] = createSignal(0)
  const [enabled, setEnabled] = createSignal(false)
  const [name, setName] = createSignal('')
  const [choice, setChoice] = createSignal('Polite')
  const [level, setLevel] = createSignal(40)
  const [sliderFocused, setSliderFocused] = createSignal(false)
  const [dialogOpen, setDialogOpen] = createSignal(false)
  const changeLevel = (value: number) => setLevel(Math.max(0, Math.min(100, value)))

  return <column width="fill" gap={18}>
    <text role="heading" level={2} text="Accessibility laboratory"
      color={props.theme.foreground} font_size={21} weight={600} />
    <text text="Use Tab to follow the focus, then Enter or Space to activate a control. Inspect its name, role and state with a screen reader."
      color={props.theme.muted} font_size={14} width="fill" />

    <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border}
      border_width={1} radius={12}>
      <column width="fill" gap={12} padding={18}>
        <text role="heading" level={3} text="Buttons and announcements"
          color={props.theme.foreground} font_size={17} weight={600} />
        <row wrap gap={10}>
          <Button id="a11y-action" label="Run action" theme={props.theme} kind="primary"
            onClick={() => setCount((value) => value + 1)} />
          <Button id="a11y-disabled" label="Unavailable" theme={props.theme} kind="outline"
            disabled onClick={() => {}} />
          <Button id="a11y-busy" label="Working" theme={props.theme} kind="secondary"
            busy onClick={() => {}} />
        </row>
        <text role="status" live="polite" text={`Action activated ${count()} times`}
          color={props.theme.foreground} font_size={13} />
      </column>
    </rectangle>

    <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border}
      border_width={1} radius={12}>
      <column width="fill" gap={14} padding={18}>
        <text role="heading" level={3} text="Forms and states"
          color={props.theme.foreground} font_size={17} weight={600} />
        <InputField id="a11y-name" label="Your name" showLabel theme={props.theme}
          value={name()} placeholder="Type a name" onChange={setName} />
        <Select id="a11y-mode" label="Announcement mode" theme={props.theme}
          options={['Polite', 'Assertive']} value={choice()} onChange={setChoice} />
        <row wrap gap={10} align_items="center">
          <Button id="a11y-notifications" role="switch" label="Notifications"
            theme={props.theme} kind="outline" selected={enabled()}
            onClick={() => setEnabled(!enabled())} />
          <text text={enabled() ? 'On' : 'Off'} color={props.theme.muted} font_size={13} />
        </row>
      </column>
    </rectangle>

    <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border}
      border_width={1} radius={12}>
      <column width="fill" gap={12} padding={18}>
        <text role="heading" level={3} text="Custom slider and media"
          color={props.theme.foreground} font_size={17} weight={600} />
        <text key="a11y-volume-help" text="Use a screen reader's increase, decrease or set value action."
          color={props.theme.muted} font_size={13} />
        <focusScope role="slider" accessible_name="Volume" described_by="a11y-volume-help"
          numeric_value={level()} minimum_value={0} maximum_value={100} value_step={10}
          orientation="horizontal" focusable can_increment can_decrement can_set_value
          onFocus={() => setSliderFocused(true)} onBlur={() => setSliderFocused(false)}
          onSemanticAction={({ action, value }) => {
            if (action === 'increment') changeLevel(level() + 10)
            if (action === 'decrement') changeLevel(level() - 10)
            if (action === 'set_value' && typeof value === 'number') changeLevel(value)
          }}
          width="fill" height={48}>
          <rectangle width="fill" height={48} radius={8} background={props.theme.surfaceRaised}
          border_color={sliderFocused() ? props.theme.accent : props.theme.border}
          border_width={sliderFocused() ? 2 : 1}>
          <row width="fill" height="fill" padding_left={14} padding_right={14} align_items="center">
            <text text="Volume" color={props.theme.foreground} font_size={14} />
            <container grow={1} />
            <text text={`${level()}%`} color={props.theme.accent} font_size={14} weight={600} />
          </row>
          </rectangle>
        </focusScope>
        <row wrap gap={8}>
          <Button id="a11y-decrease" label="Decrease volume" theme={props.theme}
            kind="outline" onClick={() => changeLevel(level() - 10)} />
          <Button id="a11y-increase" label="Increase volume" theme={props.theme}
            kind="outline" onClick={() => changeLevel(level() + 10)} />
        </row>
        <text text="The illustration and heart have alt text; the photo icon is decorative."
          color={props.theme.muted} font_size={13} width="fill" />
        <row wrap gap={12} align_items="center">
          <image source={mediaAssets['illustration/orbit.png']} alt="Colorful orbit around a planet"
            width={180} height={120} fit="contain" />
          <svg source={mediaAssets['tabler/heart.svg']} alt="Heart icon"
            color={props.theme.accent} width={48} height={48} />
          <svg source={mediaAssets['tabler/photo.svg']} alt=""
            color={props.theme.accent} width={48} height={48} />
        </row>
      </column>
    </rectangle>

    <Button id="a11y-open-dialog" label="Open dialog" theme={props.theme}
      kind="secondary" onClick={() => setDialogOpen(true)} />
    {dialogOpen() ? <focusScope role="dialog" accessible_name="Modal example" modal focusable width="fill">
      <column width="fill" gap={12} padding={16} background={props.theme.surfaceRaised}>
        <text text="This dialog has a modal role and an accessible name."
          color={props.theme.foreground} font_size={14} />
        <Button id="a11y-close-dialog" label="Close dialog" theme={props.theme}
          kind="primary" onClick={() => setDialogOpen(false)} />
      </column>
    </focusScope> : null}
  </column>
}
