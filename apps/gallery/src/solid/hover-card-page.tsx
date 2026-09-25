import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Button } from '../../../../packages/widgets/src/solid/button'
import { HoverCard } from '../../../../packages/widgets/src/solid/hover-card'

/** Demonstrates a rich, non-modal card that stays open between its trigger and content. */
export function HoverCardPage(props: { theme: Palette }): JSX.Element {
  const [following, setFollowing] = createSignal(false)
  return <column width="fill" gap={14}>
    <text width="fill" text="Hover or focus the profile link, then move into the card. Escape or an outside click closes it."
      color={props.theme.muted} font_size={14} />
    <HoverCard id="surface-c-hover-card" label="Argui profile" theme={props.theme} width={330}
      content={<row width="fill" gap={12} align_items="center">
        <rectangle width={44} height={44} radius={22} background={props.theme.accent}>
          <text width="fill" height="fill" text="A" color={props.theme.accentText}
            font_size={18} weight={700} text_align="center" />
        </rectangle>
        <column width="fill" gap={5}>
          <text text="@argui" color={props.theme.foreground} font_size={14} weight={600} />
          <text width="fill" text="A native UI toolkit for building fast cross-platform apps."
            color={props.theme.foreground} font_size={13} />
          <text text="Joined September 2026" color={props.theme.muted} font_size={11} />
        </column>
      </row>}>
      <Button id="surface-c-hover-card-trigger" label="@argui" theme={props.theme}
        kind="link" selected={following()} onClick={() => setFollowing(!following())} />
    </HoverCard>
    <text text={following() ? 'Following @argui' : 'Not following @argui'}
      color={props.theme.muted} font_size={12} />
  </column>
}
