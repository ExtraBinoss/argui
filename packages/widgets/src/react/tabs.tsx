/** @jsxImportSource @argui/react */
import { useId, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import { nextEnabledTab, type TabsOptions } from '../shared/new-controls'
import { keyName } from '../shared/types'

/** Props for a themed tab list and its selected panel. */
export type TabsProps = TabsOptions<ReactNode>

/** Renders accessible tabs with pointer and keyboard selection. */
export function Tabs(props: TabsProps): ReactElement {
  const generatedId = `argui-tabs-${useId().replaceAll(':', '')}`
  const [id] = useState(() => props.id ?? generatedId)
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = useState(() => props.items.find(
    (item) => item.value === props.defaultValue && !item.disabled,
  )?.value ?? props.items.find((item) => !item.disabled)?.value ?? '')
  const value = props.value ?? localValue
  const readOnly = props.value !== undefined && !props.onValueChange
  const activeIndex = props.items.findIndex((item) => item.value === value)
  const active = props.items[activeIndex]
  const vertical = props.orientation === 'vertical'
  const line = props.variant === 'line'
  const tabId = (itemValue: string) => `${id}-tab-${encodeURIComponent(itemValue)}`
  const panelId = (itemValue: string) => `${id}-panel-${encodeURIComponent(itemValue)}`
  const select = (index: number) => {
    const item = props.items[index]
    if (!item || item.disabled || readOnly || item.value === value) return
    if (props.value === undefined) setLocalValue(item.value)
    props.onValueChange?.(item.value)
  }
  const onKey = (payload: unknown, index: number) => {
    const key = keyName(payload)
    const forward = vertical ? 'ArrowDown' : 'ArrowRight'
    const backward = vertical ? 'ArrowUp' : 'ArrowLeft'
    if (key === forward || key === backward) {
      select(nextEnabledTab(props.items, index, key === forward ? 1 : -1))
    } else if (key === 'Home') select(nextEnabledTab(props.items, -1, 1))
    else if (key === 'End') select(nextEnabledTab(props.items, 0, -1))
  }
  const tabs = props.items.map((item, index) => <focusScope key={item.value} id={tabId(item.value)} role="tab"
    accessibleName={item.label} selected={item.value === value} controls={panelId(item.value)}
    enabled={!item.disabled && !readOnly} keyboardActivation="enterOrSpace"
    onClick={() => select(index)} onKey={(payload) => onKey(payload, index)}>
    <rectangle height={32} padding={{ start: 12, end: 12 }} radii={theme.radius - 3}
      background={!line && item.value === value ? theme.background : 'transparent'}
      hoverBackground={theme.controlHover} focusBorderColor={theme.focusRing}
      opacity={item.disabled ? 0.5 : 1}>
      <column height="100%" justifyContent="center">
        <row gap={6} alignItems="center">
          {line && vertical && item.value === value
            ? <rectangle width={2} height={18} background={theme.primary} /> : null}
          <text color={item.value === value ? theme.text : theme.textMuted}>{item.label}</text>
        </row>
        {line && !vertical && item.value === value
          ? <rectangle width="100%" height={2} background={theme.primary} /> : null}
      </column>
    </rectangle>
  </focusScope>)
  const list = <container role="tabList" accessibleName={props.accessibleName}
    orientation={vertical ? 'vertical' : 'horizontal'} activeDescendant={active ? tabId(active.value) : undefined}>
    {vertical ? <column gap={3} padding={3} background={line ? 'transparent' : theme.muted} radii={theme.radius}>
      {tabs}
    </column> : <row gap={3} padding={3} background={line ? 'transparent' : theme.muted} radii={theme.radius}>
      {tabs}
    </row>}
  </container>
  const panel = active ? <container id={panelId(active.value)} role="tabPanel"
    accessibleName={active.label} labelledBy={tabId(active.value)} grow={1} minWidth={0}>
    {active.content}
  </container> : <container grow={1} />
  return vertical
    ? <row width={props.width ?? '100%'} height={props.height} minWidth={props.minWidth}
      maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
      grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin} gap={8}>
      {list}{panel}
    </row>
    : <column width={props.width ?? '100%'} height={props.height} minWidth={props.minWidth}
      maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
      grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin} gap={8}>
      {list}{panel}
    </column>
}
