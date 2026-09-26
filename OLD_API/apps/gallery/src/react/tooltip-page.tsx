/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactButton as Button } from '../../../../packages/widgets/src/react/button'
import { ReactTooltip as Tooltip } from '../../../../packages/widgets/src/react/tooltip'

/** Demonstrates a hover- and keyboard-focus-triggered tooltip. */
export function TooltipPage(props: { theme: Palette }): ReactElement {
  const [added, setAdded] = useState(false)
  return <column width="fill" gap={14}>
    <text width="fill" text="Hover or focus the trigger to open its description. Press Escape to dismiss the anchored surface."
      color={props.theme.muted} font_size={14} />
    <Tooltip id="surface-c-tooltip" theme={props.theme}
      description={added ? 'Remove from your library' : 'Add to your library'} placement="top"
      content={<text text={added ? 'Remove from your library' : 'Add to your library'}
        color={props.theme.background} font_size={12} />}>
      <Button id="surface-c-tooltip-trigger" label={added ? 'Added to library' : 'Add to library'}
        theme={props.theme} selected={added} kind="outline" onClick={() => setAdded(!added)} />
    </Tooltip>
  </column>
}
