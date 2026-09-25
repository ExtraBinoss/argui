import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { InputEditController } from '../shared/input-edit'
import { foundationKKey, foundationKValidateQuestion, type FoundationKQuestionnaireAnswer, type FoundationKQuestionnaireProps, type FoundationKQuestionnaireStep } from '../shared/foundation-k'
import { selectionTint } from '../shared/theme'

export type QuestionnaireStep = FoundationKQuestionnaireStep
export type QuestionnaireAnswer = FoundationKQuestionnaireAnswer
export type QuestionnaireProps = FoundationKQuestionnaireProps

/** Renders a multi-step questionnaire with answer state, validation, and navigation. */
export function Questionnaire(props: QuestionnaireProps): JSX.Element {
  const [stepIndex, setStepIndex] = createSignal(0)
  const [uncontrolled, setUncontrolled] = createSignal<Readonly<Record<string, FoundationKQuestionnaireAnswer>>>(props.defaultValue ?? {})
  const [errors, setErrors] = createSignal<Readonly<Record<string, string | undefined>>>({})
  const [completed, setCompleted] = createSignal(false)
  const [choiceIndex, setChoiceIndex] = createSignal(0)
  const edits = new Map<string, InputEditController>()
  const values = () => props.value !== undefined ? props.value : uncontrolled()
  const steps = () => props.steps
  const step = () => steps()[stepIndex()]
  const answer = (current: FoundationKQuestionnaireStep | undefined = step()) => current ? values()[current.id] : undefined
  const setAnswer = (current: FoundationKQuestionnaireStep, next: FoundationKQuestionnaireAnswer) => {
    const updated = { ...values(), [current.id]: next }
    if (props.value === undefined) setUncontrolled(updated)
    props.onValueChange?.(updated)
    setErrors({ ...errors(), [current.id]: undefined })
    setCompleted(false)
  }
  const editController = (current: FoundationKQuestionnaireStep, currentValue: string) => {
    let controller = edits.get(current.id)
    if (!controller) { controller = new InputEditController(currentValue); edits.set(current.id, controller) }
    return controller
  }
  const validateCurrent = () => {
    const current = step()
    if (!current) return false
    const message = foundationKValidateQuestion(current, answer(current))
    setErrors({ ...errors(), [current.id]: message })
    return message === undefined
  }
  const finish = () => {
    setCompleted(true)
    props.onSubmit?.({ ...values() })
  }
  const next = () => {
    if (!validateCurrent()) return
    if (stepIndex() >= steps().length - 1) finish()
    else { setStepIndex(stepIndex() + 1); setChoiceIndex(0) }
  }
  const previous = () => {
    if (stepIndex() > 0) { setStepIndex(stepIndex() - 1); setChoiceIndex(0) }
  }
  const skip = () => {
    if (step()?.required !== false) return
    setErrors({ ...errors(), [step()!.id]: undefined })
    if (stepIndex() >= steps().length - 1) finish()
    else { setStepIndex(stepIndex() + 1); setChoiceIndex(0) }
  }
  const choose = (current: FoundationKQuestionnaireStep, value: string) => {
    if (current.type === 'multiple') {
      const selected = Array.isArray(answer(current)) ? [...answer(current) as readonly string[]] : []
      const nextValues = selected.includes(value) ? selected.filter((entry) => entry !== value) : [...selected, value]
      setAnswer(current, nextValues)
    } else setAnswer(current, value)
  }
  const choiceSelected = (current: FoundationKQuestionnaireStep, value: string) => {
    const currentAnswer = answer(current)
    return Array.isArray(currentAnswer) ? currentAnswer.includes(value) : currentAnswer === value
  }
  const moveRadio = (payload: unknown) => {
    const key = foundationKKey(payload)
    const current = step()
    if (!current) return
    const enabledOptions = (current.options ?? []).filter((option) => !option.disabled)
    if (enabledOptions.length === 0) return
    let nextIndex = enabledOptions.findIndex((option) => option.value === answer(current))
    if (nextIndex < 0) nextIndex = Math.min(choiceIndex(), enabledOptions.length - 1)
    if (key === 'ArrowDown' || key === 'ArrowRight') nextIndex = (nextIndex + 1) % enabledOptions.length
    else if (key === 'ArrowUp' || key === 'ArrowLeft') nextIndex = (nextIndex - 1 + enabledOptions.length) % enabledOptions.length
    else if (key === 'Home') nextIndex = 0
    else if (key === 'End') nextIndex = enabledOptions.length - 1
    else return
    setChoiceIndex(nextIndex)
    choose(current, enabledOptions[nextIndex]!.value)
  }
  const action = (label: string, primary: boolean, enabled: boolean, onClick: () => void) =>
    <focusScope role="button" accessible_name={label} enabled={enabled} focusable={true}
      keyboard_activation="enter_or_space" onClick={() => { if (enabled) onClick() }}>
      <touchArea enabled={enabled} mouse_cursor={enabled ? 'pointer' : 'not_allowed'}>
        <rectangle height={props.theme.inputHeight} background={primary ? props.theme.accent : props.theme.surfaceRaised}
          border_color={primary ? props.theme.accent : props.theme.border} border_width={1} radius={props.theme.controlRadius}
          opacity={enabled ? 1 : 0.48}>
          <row height="fill" padding_left={14} padding_right={14} align_items="center" justify_content="center">
            <text text={label} color={primary ? props.theme.accentText : props.theme.foreground}
              font_size={props.theme.controlFontSize} weight={600} />
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
  const width = Number.isFinite(props.width) ? Math.max(300, props.width!) : 520
  const current = () => step()
  if (steps().length === 0) return <column width={width} gap={8}>
    <text text={props.label} color={props.theme.foreground} font_size={16} weight={700} />
    <text text="No questions are configured." color={props.theme.muted} font_size={13} />
  </column>
  const currentAnswer = () => current() ? answer(current()) : undefined
  const textValue = () => typeof currentAnswer() === 'string' ? currentAnswer() as string : ''
  const progress = () => Math.round((stepIndex() + 1) / steps().length * 100)
  const enabledOptions = () => current()?.type === 'single' ? (current()?.options ?? []).filter((option) => !option.disabled) : []
  const activeRadio = () => enabledOptions().find((option) => option.value === currentAnswer())
    ?? enabledOptions()[Math.min(choiceIndex(), Math.max(0, enabledOptions().length - 1))]
  const error = () => current() ? errors()[current()!.id] : undefined
  return <focusScope key={`${props.id}-questionnaire`} role="group" accessible_name={props.label}>
    <rectangle width={width} background={props.theme.surface} border_color={props.theme.border}
      border_width={1} radius={props.theme.overlayRadius}>
      <column width="fill" gap={18} padding={props.theme.overlayPadding}>
        <column width="fill" gap={8}>
          <row width="fill" align_items="center" justify_content="space-between" gap={8}>
            <text text={props.label} color={props.theme.foreground} font_size={16} weight={700} />
            <text text={`${stepIndex() + 1} of ${steps().length}`} color={props.theme.muted} font_size={11} />
          </row>
          <focusScope role="progress" accessible_name="Questionnaire progress" numeric_value={stepIndex() + 1}
            minimum_value={0} maximum_value={steps().length} focusable={false}>
            <rectangle width={160} height={6} background={props.theme.surfaceRaised} radius={3}>
              <rectangle width={160 * progress() / 100} height={6} background={props.theme.accent} radius={3} />
            </rectangle>
          </focusScope>
        </column>
        {completed() ? <column gap={8}>
          <text text={props.completedLabel ?? 'Your responses were submitted.'} color={props.theme.accent} font_size={14}
            role="status" live="polite" />
        </column> : current() ? <column width="fill" gap={12}>
          <column gap={4}>
            <text text={current()!.title} color={props.theme.foreground} font_size={15} weight={600} />
            {current()!.description ? <text text={current()!.description!} color={props.theme.muted} font_size={12} /> : null}
          </column>
          {current()!.type === 'text' ? <rectangle width="fill" height={props.theme.inputHeight}
            background={props.theme.surface} border_color={error() ? props.theme.destructive : props.theme.border}
            border_width={1} radius={props.theme.controlRadius}>
            <textInput key={`${props.id}-${current()!.id}`} role="text_input" width="fill" height="fill" clip={true}
              value={textValue()} placeholder={current()!.placeholder ?? 'Type your answer'} label={current()!.title}
              description={current()!.description} required={current()!.required !== false} invalid={!!error()}
              background="#00000000" text_color={props.theme.foreground} placeholder_color={props.theme.muted}
              caret_color={props.theme.accent} selection_color={selectionTint(props.theme.accent)}
              onEdit={(payload) => editController(current()!, textValue()).apply(payload, textValue(), (nextValue) => setAnswer(current()!, nextValue))} />
          </rectangle> : current()!.type === 'single' ? <focusScope key={`${props.id}-${current()!.id}-choices`}
            role="radio_group" accessible_name={current()!.title} accessible_description={current()!.description}
            orientation="vertical" required={current()!.required !== false} invalid={!!error()}
            focusable={enabledOptions().length > 0} enabled={enabledOptions().length > 0}
            active_descendant={activeRadio() ? `${props.id}-${current()!.id}-choice-${activeRadio()!.value}` : undefined}
            onKey={moveRadio}>
            <column width="fill" gap={7}>
              {(current()?.options ?? []).map((option) => {
                const selected = choiceSelected(current()!, option.value)
                const active = activeRadio()?.value === option.value
                return <focusScope key={`${props.id}-${current()!.id}-choice-${option.value}`} role="radio_button"
                  accessible_name={option.label} accessible_description={option.description}
                  selected={selected} checked_state={selected ? 'checked' : 'unchecked'} enabled={!option.disabled}
                  focusable={false} focus_on_tab_navigation={false} onClick={() => { if (!option.disabled) choose(current()!, option.value) }}>
                  <touchArea enabled={!option.disabled} mouse_cursor={option.disabled ? 'not_allowed' : 'pointer'}>
                    <rectangle width="fill" height={48} background={selected ? props.theme.surfaceHover : props.theme.surface}
                      border_color={error() ? props.theme.destructive : active ? props.theme.accent : props.theme.border}
                      border_width={selected || active ? 2 : 1} radius={props.theme.controlRadius} opacity={option.disabled ? 0.48 : 1}>
                      <row width="fill" height="fill" gap={10} padding_left={12} padding_right={12} align_items="center">
                        <rectangle width={16} height={16} radius={8} background={props.theme.surface}
                          border_color={selected ? props.theme.accent : props.theme.border} border_width={selected ? 5 : 1} />
                        <column gap={2}>
                          <text text={option.label} color={props.theme.foreground} font_size={13} />
                          {option.description ? <text text={option.description} color={props.theme.muted} font_size={11} /> : null}
                        </column>
                      </row>
                    </rectangle>
                  </touchArea>
                </focusScope>
              })}
            </column>
          </focusScope> : <focusScope key={`${props.id}-${current()!.id}-checkboxes`} role="group"
            accessible_name={current()!.title} accessible_description={current()!.description}
            required={current()!.required !== false} invalid={!!error()}>
            <column width="fill" gap={7}>
            {(current()?.options ?? []).map((option) => {
              const selected = choiceSelected(current()!, option.value)
              return <focusScope key={`${props.id}-${current()!.id}-choice-${option.value}`} role="check_box"
                accessible_name={option.label} accessible_description={option.description}
                selected={selected} checked_state={selected ? 'checked' : 'unchecked'} enabled={!option.disabled}
                keyboard_activation="enter_or_space" onClick={() => { if (!option.disabled) choose(current()!, option.value) }}>
                <touchArea enabled={!option.disabled} mouse_cursor={option.disabled ? 'not_allowed' : 'pointer'}>
                  <rectangle width="fill" height={48} background={selected ? props.theme.surfaceHover : props.theme.surface}
                    border_color={error() ? props.theme.destructive : selected ? props.theme.accent : props.theme.border}
                    border_width={selected ? 2 : 1} radius={props.theme.controlRadius} opacity={option.disabled ? 0.48 : 1}>
                    <row width="fill" height="fill" gap={10} padding_left={12} padding_right={12} align_items="center">
                      <rectangle width={16} height={16} radius={4} background={selected ? props.theme.accent : props.theme.surface}
                        border_color={selected ? props.theme.accent : props.theme.border} border_width={1}>
                        {selected ? <text text="✓" width="fill" height="fill" color={props.theme.accentText}
                          font_size={11} weight={700} text_align="center" accessible_hidden={true} /> : null}
                      </rectangle>
                      <column gap={2}>
                        <text text={option.label} color={props.theme.foreground} font_size={13} />
                        {option.description ? <text text={option.description} color={props.theme.muted} font_size={11} /> : null}
                      </column>
                    </row>
                  </rectangle>
                </touchArea>
              </focusScope>
            })}
            </column>
          </focusScope>}
          {error() ? <text text={error()!} color={props.theme.destructive} font_size={12} role="alert" live="assertive" /> : null}
        </column> : null}
        {!completed() ? <row width="fill" gap={8} align_items="center">
          {stepIndex() > 0 ? action(props.previousLabel ?? 'Previous', false, true, previous) : <container width="fill" />}
          {current()?.required === false ? action(props.skipLabel ?? 'Skip', false, true, skip) : null}
          <container grow={1} />
          {action(stepIndex() === steps().length - 1 ? props.submitLabel ?? 'Submit' : props.nextLabel ?? 'Next', true, true, next)}
        </row> : null}
      </column>
    </rectangle>
  </focusScope>
}
