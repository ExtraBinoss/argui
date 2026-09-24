import { Button, type Palette } from '@argui/widgets/solid'
import { useI18n } from '@argui/i18n/solid'
import type { JSX } from '@argui/solid/jsx-runtime'

/** Demonstrates Fluent formatting and a reactive native locale switch in Solid. */
export function I18nPage(props: { theme: Palette }): JSX.Element {
  const i18n = useI18n()
  const targetLocale = () => i18n.locale().startsWith('fr') ? 'en-US' : 'fr'
  const switchMessage = () => i18n.locale().startsWith('fr') ? 'switchToEnglish' : 'switchToFrench'
  return (
    <column width="fill" gap={14}>
      <text text={i18n.tr('welcome', { name: 'Ada' })} color={props.theme.foreground} font_size={20} />
      <text text={i18n.tr('currentLocale', { locale: i18n.locale() })}
        color={props.theme.muted} font_size={14} />
      <Button id="i18n-switch-locale" label={i18n.tr(switchMessage())} theme={props.theme}
        kind="primary" onClick={() => i18n.selectLocale(targetLocale())} />
    </column>
  )
}
