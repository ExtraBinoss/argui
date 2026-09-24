/** @jsxImportSource @argui/react */
import { Button, type Palette } from '@argui/widgets/react'
import { useI18n } from '@argui/i18n/react'
import type { ReactElement } from 'react'

/** Demonstrates Fluent formatting and a reactive native locale switch in React. */
export function ReactI18nPage(props: { theme: Palette }): ReactElement {
  const i18n = useI18n()
  const isFrench = i18n.locale.startsWith('fr')
  const targetLocale = isFrench ? 'en-US' : 'fr'
  const switchMessage = isFrench ? 'switchToEnglish' : 'switchToFrench'
  return (
    <column width="fill" gap={14}>
      <text text={i18n.tr('welcome', { name: 'Ada' })} color={props.theme.foreground} font_size={20} />
      <text text={i18n.tr('currentLocale', { locale: i18n.locale })}
        color={props.theme.muted} font_size={14} />
      <Button id="react-i18n-switch-locale" label={i18n.tr(switchMessage)} theme={props.theme}
        kind="primary" onClick={() => i18n.selectLocale(targetLocale)} />
    </column>
  )
}
