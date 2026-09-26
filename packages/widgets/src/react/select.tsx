/** @jsxImportSource @argui/react */
import { useId, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import { keyName, nextEnabledOption, type SelectOptions } from '../shared/types'

/** Props for the native React Select. */
export type SelectProps = SelectOptions & { leading?: ReactNode; trailing?: ReactNode }

/** Renders a keyboard-accessible native option list with controlled or local value. */
export function Select(props: SelectProps): ReactElement {
  const generatedId = `argui-select-${useId().replaceAll(':', '')}`
  const id = props.id ?? generatedId
  const popupId = `${id}-popup`
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = useState(props.defaultValue ?? '')
  const [localOpen, setLocalOpen] = useState(props.defaultOpen ?? false)
  const [activeIndex, setActiveIndex] = useState(-1)
  const value = props.value ?? localValue
  const expanded = props.open ?? localOpen
  const controlledReadOnly = props.value !== undefined && !props.onValueChange
  const fixedOpen = props.open !== undefined && !props.onOpenChange
  const disabled = !!props.disabled || controlledReadOnly
  const selectedIndex = props.options.findIndex((option) => option.value === value)
  const currentIndex = activeIndex >= 0 && activeIndex < props.options.length ? activeIndex : selectedIndex

  const setOpen = (next: boolean) => {
    if (props.open === undefined && !fixedOpen) setLocalOpen(next)
    props.onOpenChange?.(next)
  }
  const showOptions = () => {
    if (disabled || fixedOpen) return
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
  const onKey = (payload: unknown) => {
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
  const width = props.width ?? 240
  const shadcn = props.variant === 'shadcn'
  const rowHeight = shadcn ? 28 : 36
  const rowCount = Math.max(1, props.options.length + Number(shadcn))
  const contentHeight = 8 + rowCount * rowHeight + (rowCount - 1) * 2 + (shadcn ? 26 : 0)
  const optionHeight = Math.min(256, contentHeight)
  // The popup's selected row overlays the center of the 32 px trigger when every row fits.
  const placementOffset = shadcn && contentHeight <= 256
    ? -(16 + 1 + 4 + 24 + 2 + Math.max(0, selectedIndex + 1) * 30 + 14)
    : undefined

  return <column width={props.width ?? '100%'} height={props.height}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin} gap={theme.spacing}>
    {!shadcn ? <text color={theme.textMuted} fontSize={12}>{props.label}</text> : null}
    <focusScope
      id={id}
      role="comboBox"
      accessibleName={props.label}
      accessibleValue={selected?.label ?? ''}
      enabled={!disabled && !fixedOpen}
      expandable={true}
      expanded={expanded}
      controls={popupId}
      hasPopup="listBox"
      activeDescendant={expanded && currentIndex >= 0
        ? `${id}-option-${encodeURIComponent(props.options[currentIndex]!.value)}` : undefined}
      keyboardActivation="none"
      onClick={() => expanded ? setOpen(false) : showOptions()}
      onKey={onKey}
    >
      <rectangle
        width={width}
        height={shadcn ? 32 : 40}
        padding={theme.spacing}
        background={theme.surface}
        border={{ width: 1, color: theme.border }}
        radii={theme.radius}
        focusBorderColor={theme.focusRing}
        opacity={props.disabled ? 0.55 : 1}
      >
        <row width="100%" height="100%" gap={theme.spacing} alignItems="center">
          {props.leading}
          <container grow={1} minWidth={0}>
            <text color={selected ? theme.text : theme.textMuted}>
              {selected?.label ?? props.placeholder ?? 'Choose an option'}
            </text>
          </container>
          {props.trailing}
        </row>
      </rectangle>
    </focusScope>
    {expanded ? <popupWindow
      id={popupId}
      role="listBox"
      accessibleName={props.label}
      anchor={id}
      placement="bottomStart"
      placementOffset={placementOffset}
      width={width}
      windowLayer="popover"
      dismissPolicy="outsidePointerOrEscape"
      containment="trap"
      initialFocus="first"
      restoreFocus={true}
      onDismiss={() => setOpen(false)}
    >
      <rectangle
        width="100%"
        background={theme.surface}
        border={{ width: theme.overlayBorderWidth, color: theme.border }}
        radii={theme.overlayRadius}
        shadow={{ offsetY: theme.overlayShadowOffsetY, blur: theme.overlayShadowBlur, color: theme.overlayShadowColor }}
      >
        <scrollView width="100%" height={optionHeight}>
          <column width="100%" gap={2} padding={4}>
            {shadcn ? <container height={24} padding={{ start: 6, top: 3 }}>
              <text color={theme.textMuted} fontSize={12}>{props.label}</text>
            </container> : null}
            {shadcn ? <focusScope
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
              <touchArea enabled={!disabled} mouseCursor={disabled ? 'notAllowed' : 'pointer'}>
                <rectangle width="100%" height={rowHeight} padding={{ start: 6, end: 6 }}
                  background={!selected ? theme.surfaceHover : theme.surface}
                  hoverBackground={theme.controlHover} radii={theme.radius}>
                  <row width="100%" height="100%" alignItems="center">
                    <container grow={1} minWidth={0}><text color={theme.text}>{props.placeholder ?? 'Choose an option'}</text></container>
                    {!selected ? <text color={theme.text}>✓</text> : null}
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
                onClick={() => choose(index)}
              >
                <touchArea
                  enabled={!option.disabled && !disabled}
                  mouseCursor={option.disabled || disabled ? 'notAllowed' : 'pointer'}
                  onPointerEnter={() => { if (!option.disabled) setActiveIndex(index) }}
                >
                  <rectangle
                    width="100%"
                    height={rowHeight}
                    padding={shadcn ? { start: 6, end: 6 } : theme.spacing}
                    background={active || value === option.value ? theme.surfaceHover : theme.surface}
                    hoverBackground={theme.controlHover}
                    radii={theme.radius}
                    opacity={option.disabled ? 0.5 : 1}
                  >
                    <row width="100%" height="100%" alignItems="center">
                      <container grow={1} minWidth={0}><text color={theme.text}>{option.label}</text></container>
                      {shadcn && value === option.value ? <text color={theme.text}>✓</text> : null}
                    </row>
                  </rectangle>
                </touchArea>
              </focusScope>
            }) : !shadcn ? <text color={theme.textMuted}>No options</text> : null}
          </column>
        </scrollView>
      </rectangle>
    </popupWindow> : null}
  </column>
}
