/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button } from '@argui/widgets/react'
import type { Palette } from '@argui/widgets/react'
import { ReactBadge as Badge } from '../../../../packages/widgets/src/react/badge'
import { ReactCard as Card, ReactCardAction as CardAction, ReactCardContent as CardContent, ReactCardDescription as CardDescription, ReactCardFooter as CardFooter, ReactCardHeader as CardHeader, ReactCardTitle as CardTitle } from '../../../../packages/widgets/src/react/card'

/** Builds a card composition with header action, live status, and footer action. */
export function CardPage(props: { theme: Palette }): ReactElement {
  const [deployments, setDeployments] = useState(0)
  const [opened, setOpened] = useState(false)
  return <column width="fill" gap={14}>
    <text text="Cards group related content and actions on a themed surface." color={props.theme.muted} font_size={14} />
    <Card theme={props.theme}>
      <CardHeader theme={props.theme} action={<CardAction>
        <Button id="card-open-report" label={opened ? 'Report open' : 'Open report'} theme={props.theme}
          kind="ghost" selected={opened} onClick={() => setOpened((value) => !value)} />
      </CardAction>}>
        <CardTitle theme={props.theme} text="Production deployment" />
        <CardDescription theme={props.theme} text="Release controls for the default environment." />
      </CardHeader>
      <CardContent theme={props.theme}>
        <row width="fill" justify_content="space_between" align_items="center" gap={8}>
          <text text="Environment" color={props.theme.muted} font_size={13} />
          <Badge theme={props.theme} label="Healthy" variant="secondary" size="compact" />
        </row>
        <text text={`Deployments today: ${deployments}`} color={props.theme.foreground} font_size={14} />
      </CardContent>
      <CardFooter theme={props.theme} justify="space_between">
        <text text={opened ? 'Report selected' : 'Ready to deploy'} color={props.theme.muted} font_size={12} />
        <Button id="card-deploy" label="Deploy" theme={props.theme} kind="primary"
          onClick={() => setDeployments((count) => count + 1)} />
      </CardFooter>
    </Card>
    <Card theme={props.theme} elevated={false} padding={props.theme.controlPadding}>
      <CardContent theme={props.theme}>
        <text text="Flat compact card" color={props.theme.foreground} font_size={14} weight={600} />
        <text text="The same surface can be used without elevation." color={props.theme.muted} font_size={13} />
      </CardContent>
    </Card>
  </column>
}
