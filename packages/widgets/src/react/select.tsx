/** @jsxImportSource @argui/react */
import { useId, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import { keyName, nextEnabledOption, type SelectOptions } from '../shared/types'
import type { AssetRef } from '@argui/host'
import { ScrollShadow } from './scroll-shadow'
import { selectMarqueePeriod, selectMenuOffset } from '../shared/select-layout'

/** Props for the native React Select. */
export type SelectProps = SelectOptions & {
  /** Application vector for the selected option; avoids dependence on font glyph coverage. */
  selectedIcon?: AssetRef
  leading?: ReactNode
  /** App-supplied chevron, rotated while the option list is open. */
  trailing?: ReactNode
}

/** Renders a keyboard-accessible native option list with controlled or local value. */
export function Select(props: SelectProps): ReactElement {
  const generatedId = `argui-select-${useId().replaceAll(':', '')}`
  const [id] = useState(() => props.id ?? generatedId)
  const popupId = `${id}-popup`
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = useState(props.defaultValue ?? '')
  const [localOpen, setLocalOpen] = useState(props.defaultOpen ?? false)
  const [activeIndex, setActiveIndex] = useState(-1)
  const [triggerWidth, setTriggerWidth] = useState<number>()
  const [triggerHeight, setTriggerHeight] = useState<number>()
  const value = props.value ?? localValue
  const expanded = props.open ?? localOpen
  const controlledReadOnly = props.value !== undefined && !props.onValueChange
  const disabled = !!props.disabled || controlledReadOnly
  const selectedIndex = props.options.findIndex((option) => option.value === value)
  const currentIndex = activeIndex >= 0 && activeIndex < props.options.length ? activeIndex : selectedIndex

  const setOpen = (next: boolean) => {
    if (props.open === undefined) setLocalOpen(next)
    props.onOpenChange?.(next)
  }
  const showOptions = () => {
    if (disabled) return
    const selected = selectedIndex
    setActiveIndex(selected >= 0 && !props.options[selected]?.disabled
      ? selected : nextEnabledOption(props.options, -1, 1))
    setOpen(true)
  }
  const choose = (index: number) => {
    const option = props.options[index]
    if (!option || option.disabled || disabled) return
    if (props.value === undefined) setLocalValue(option.value)
    props.onValueChange?.(option.value)
    setOpen(false)
  }
  const measureTrigger = (payload: unknown) => {
    const { width, height } = (payload as { width?: number; height?: number } | null) ?? {}
    if (typeof width === 'number' && Number.isFinite(width) && width > 0) setTriggerWidth(width)
    if (typeof height === 'number' && Number.isFinite(height) && height > 0) setTriggerHeight(height)
  }
  const onKey = (payload: unknown) => {
    measureTrigger(payload)
    if (disabled) return
    const key = keyName(payload)
    if (!key) return
    if (key === 'Escape') setOpen(false)
    else if (key === 'Enter' || key === ' ') {
      if (expanded) choose(currentIndex)
      else showOptions()
    } else if (key === 'ArrowDown' || key === 'ArrowUp') {
      const next = nextEnabledOption(props.options,
        currentIndex < 0 ? (key === 'ArrowDown' ? -1 : 0) : currentIndex,
        key === 'ArrowDown' ? 1 : -1)
      if (next >= 0) { setActiveIndex(next); setOpen(true) }
    } else if (key === 'Home' || key === 'End') {
      const next = key === 'Home'
        ? nextEnabledOption(props.options, -1, 1)
        : nextEnabledOption(props.options, 0, -1)
      if (next >= 0) { setActiveIndex(next); setOpen(true) }
    }
  }

  const selected = props.options.find((option) => option.value === value)
  const width = props.width ?? theme.selectWidth
  const popupWidth = props.contentWidth ?? triggerWidth ?? (typeof width === 'number' ? width : theme.selectWidth)
  const shadcn = props.variant === 'shadcn'
  const clearable = shadcn && props.allowClear !== false
  const shadowInset = Math.ceil(theme.overlayShadowBlur + Math.abs(theme.overlayShadowOffsetY))
  const labelWidth = Math.max(1, (typeof popupWidth === 'number' ? popupWidth : theme.selectWidth)
    - (shadcn ? 38 : 10 + theme.spacing * 2))
  const rowHeight = shadcn ? theme.selectCompactRowHeight : theme.selectRowHeight
  const rowCount = Math.max(1, props.options.length + Number(clearable))
  const contentHeight = 8 + rowCount * rowHeight + (rowCount - 1) * 2 + (shadcn ? 26 : 0)
  const optionHeight = Math.min(theme.selectMaxPopupHeight, contentHeight)
  const placementOffset = shadcn
    ? selectMenuOffset(triggerHeight ?? theme.selectCompactHeight, rowHeight,
      selectedIndex + Number(clearable), optionHeight, shadowInset, theme.overlayBorderWidth)
    : 4 - shadowInset
  const initialFocus = selected ? `${id}-option-${encodeURIComponent(selected.value)}`
    : clearable ? `${id}-placeholder` : 'first'

  return <column width={width} height={props.height}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin} gap={theme.spacing}>
    {!shadcn ? <text color={theme.textMuted} fontSize={theme.fieldLabelSize}>{props.label}</text> : null}
    <focusScope
      id={id}
      role="comboBox"
      accessibleName={props.label}
      accessibleValue={selected?.label ?? ''}
      enabled={!disabled}
      mouseCursor={disabled ? 'notAllowed' : 'pointer'}
      expandable={true}
      expanded={expanded}
      controls={popupId}
      hasPopup="listBox"
      activeDescendant={expanded && currentIndex >= 0
        ? `${id}-option-${encodeURIComponent(props.options[currentIndex]!.value)}` : undefined}
      keyboardActivation="none"
      onClick={event => { measureTrigger(event); expanded ? setOpen(false) : showOptions() }}
      onKey={onKey}
    >
      <rectangle
        width="100%"
        height={shadcn ? theme.selectCompactHeight : theme.inputHeight}
        padding={{ start: theme.spacing, end: theme.spacing }}
        background={theme.surface}
        border={{ width: 1, color: theme.border }}
        radii={theme.radius}
        focusBorderColor={theme.focusRing}
        opacity={props.disabled ? 0.55 : 1}
      >
        <row width="100%" height="100%" gap={theme.spacing} alignItems="center">
          {props.leading}
          <row width={0} grow={1} minWidth={0} height={20} alignItems="center" clip={true}>
            <text width="100%" noWrap={true} textOverflow="ellipsis" color={selected ? theme.text : theme.textMuted} fontSize={13}>
              {selected?.label ?? props.placeholder ?? 'Choose an option'}
            </text>
          </row>
          {props.trailing ? <container id={`${id}-chevron`} shrink={0}
            rotation={expanded ? 180 : 0} transitionMs={180}
            transitionTimingFunction="cubic-bezier(0.2, 0, 0, 1)">{props.trailing}</container> : null}
        </row>
      </rectangle>
    </focusScope>
    {expanded ? <popupWindow
      allowOutsideWindow={props.allowOutsideWindow}
      id={popupId}
      role="listBox"
      accessibleName={props.label}
      anchor={id}
      placement="bottomStart"
      anchorWidth={props.contentWidth === undefined ? "matchAnchor" : "content"}
      anchorWidthOffset={2 * shadowInset}
      placementOffset={placementOffset}
      placementCrossOffset={-shadowInset}
      width={(typeof popupWidth === 'number' ? popupWidth : theme.selectWidth) + 2 * shadowInset}
      windowLayer="popover"
      dismissPolicy="outsidePointerOrEscape"
      containment="trap"
      initialFocus={initialFocus}
      restoreFocus={true}
      openingMs={150}
      openingScale={0.98}
      openingTranslateY={shadcn ? 0 : -4}
      onDismiss={() => setOpen(false)}
    >
      <container width="100%" padding={shadowInset}>
      <rectangle
        width="100%"
        background={theme.popover}
        clip={true}
        border={{ width: theme.overlayBorderWidth, color: theme.outlineBorder }}
        radii={theme.overlayRadius}
        shadow={{ offsetY: theme.overlayShadowOffsetY, blur: theme.overlayShadowBlur, color: theme.overlayShadowColor }}
      >
        <ScrollShadow width="100%" height={optionHeight} scrollbarEndInset={0}>
          <column width="100%" minWidth={0} maxWidth="100%" gap={2} padding={4}>
            {shadcn ? <row height={24} padding={{ start: 6 }} alignItems="center">
              <text color={theme.textMuted} fontSize={theme.fieldLabelSize}>{props.label}</text>
            </row> : null}
            {clearable ? <focusScope
              id={`${id}-placeholder`}
              role="option"
              accessibleName={props.placeholder ?? 'Choose an option'}
              selected={!selected}
              enabled={!disabled}
              keyboardActivation="enterOrSpace"
              onClick={() => {
                if (disabled) return
                if (props.value === undefined) setLocalValue('')
                props.onValueChange?.('')
                setOpen(false)
              }}
            >
              <touchArea enabled={!disabled} mouseCursor={disabled ? 'notAllowed' : 'pointer'}
                onPointerEnter={() => setActiveIndex(-1)} onPointerLeave={() => setActiveIndex(-1)}>
                <rectangle width="100%" height={rowHeight} padding={{ start: 6, end: 6 }}
                  background={!selected ? theme.surfaceHover : 'transparent'}
                  hoverBackground={theme.controlHover} radii={theme.radius}>
                  <row width="100%" height="100%" alignItems="center">
                    <container width={0} grow={1} minWidth={0} clip={true}><text width="100%" noWrap={true} textOverflow="ellipsis" color={theme.text} fontSize={13}>{props.placeholder ?? 'Choose an option'}</text></container>
                    {!selected ? <>{props.selectedIcon ? <svg source={props.selectedIcon} width={14} height={14} color={theme.text} /> : <text color={theme.text}>✓</text>}</> : null}
                  </row>
                </rectangle>
              </touchArea>
            </focusScope> : null}
            {props.options.length ? props.options.map((option, index) => {
              const optionId = `${id}-option-${encodeURIComponent(option.value)}`
              const active = index === currentIndex
              return <focusScope
                id={optionId}
                key={option.value}
                role="option"
                accessibleName={option.label}
                selected={value === option.value}
                enabled={!option.disabled && !disabled}
                keyboardActivation="enterOrSpace"
                mouseCursor={option.disabled || disabled ? 'notAllowed' : 'pointer'}
                onClick={() => choose(index)}
              >
                  <rectangle
                    width="100%"
                    height={rowHeight}
                    padding={shadcn ? { start: 6, end: 6 } : theme.spacing}
                    background={active || value === option.value ? theme.surfaceHover : 'transparent'}
                    hoverBackground={theme.controlHover}
                    radii={theme.radius}
                    opacity={option.disabled ? 0.5 : 1}
                  >
                    <row width="100%" height="100%" alignItems="center">
                      <container width={0} grow={1} minWidth={0}>
                        <SelectLabel label={option.label} viewportWidth={labelWidth} onHover={() => setActiveIndex(-1)} />
                      </container>
                      {shadcn ? <container width={16} shrink={0}>
                        {value === option.value ? <>{props.selectedIcon ? <svg source={props.selectedIcon} width={14} height={14} color={theme.text} /> : <text color={theme.text}>✓</text>}</> : null}
                      </container> : null}
                    </row>
                  </rectangle>
              </focusScope>
            }) : !shadcn ? <text color={theme.textMuted}>No options</text> : null}
          </column>
        </ScrollShadow>
      </rectangle>
      </container>
    </popupWindow> : null}
  </column>
}

/** Measures an option's actual text box on hover and moves it with a native loop. */
function SelectLabel(props: { label: string; viewportWidth: number; onHover: () => void }): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [overflow, setOverflow] = useState(0)
  const [hovered, setHovered] = useState(false)
  const running = hovered && overflow > 1
  return <ScrollShadow width="100%" height={20} orientation="horizontal" size={12} scrollbarVisible={false}>
    <row height={20}>
      <container shrink={0}>
      <touchArea
        onPointerEnter={event => {
          setOverflow(Math.max(0, (event.width ?? 0) - props.viewportWidth))
          setHovered(true); props.onHover()
        }} onPointerLeave={() => { setHovered(false); props.onHover() }}>
        <rectangle height={20} loopMs={running ? selectMarqueePeriod(overflow) : undefined}
          loopTranslateX={running ? -overflow : undefined} loopHold={running ? true : undefined}>
          <text noWrap={true} color={theme.text} fontSize={13} lineHeight={20}>{props.label}</text>
        </rectangle>
      </touchArea>
      </container>
    </row>
  </ScrollShadow>
}
