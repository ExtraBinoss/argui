/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactQuestionnaire as Questionnaire, type ReactQuestionnaireAnswer, type ReactQuestionnaireStep } from '../../../../packages/widgets/src/react/questionnaire'

const steps: readonly ReactQuestionnaireStep[] = [
  {
    id: 'team', title: 'Which team are you on?', description: 'This helps route your feedback.', type: 'single',
    options: [
      { value: 'design', label: 'Design', description: 'Product and visual design' },
      { value: 'engineering', label: 'Engineering', description: 'Application and platform work' },
      { value: 'operations', label: 'Operations', description: 'Support and internal operations' },
    ],
  },
  {
    id: 'areas', title: 'Which areas should we improve?', type: 'multiple',
    description: 'Choose all that apply.',
    options: [
      { value: 'navigation', label: 'Navigation' },
      { value: 'reports', label: 'Reports and dashboards' },
      { value: 'notifications', label: 'Notifications' },
    ],
  },
  {
    id: 'note', title: 'Anything else we should know?', description: 'Optional; include a few details if you have them.',
    type: 'text', required: false, placeholder: 'Share an example…',
    validate: (answer) => typeof answer === 'string' && answer.trim().length > 0 && answer.trim().length < 12
      ? 'Please add at least 12 characters, or skip this question.' : undefined,
  },
]

/** Demonstrates radio and checkbox answers, optional skipping, text validation, and submit feedback. */
export function QuestionnairePage(props: { theme: Palette }): ReactElement {
  const [submitted, setSubmitted] = useState('')
  return <column width="fill" gap={12}>
    <text text="Try continuing without choosing a team or improvement area to see validation. You can skip the optional note, or enter at least 12 characters."
      color={props.theme.muted} font_size={13} />
    <Questionnaire id="foundation-k-feedback" label="Product feedback" theme={props.theme} steps={steps}
      onSubmit={(answers: Readonly<Record<string, ReactQuestionnaireAnswer>>) => {
        const team = typeof answers.team === 'string' ? answers.team : 'not specified'
        const areas = Array.isArray(answers.areas) ? answers.areas.join(', ') : 'none'
        setSubmitted(`Submitted for ${team}; selected areas: ${areas}.`)
      }} />
    {submitted ? <text text={submitted} color={props.theme.accent} font_size={13} role="status" live="polite" /> : null}
  </column>
}
