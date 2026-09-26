// Generated from argui-schema by packages/react/scripts/generate-jsx.mjs.
import type { ReactNode, Ref } from 'react'
import type { NativeHandle, NativeEventPayload, AssetRef, DimensionValue, ConstraintValue, GridTracksValue, ContainerRuleValue, InsetsValue, PositionInsetsValue, RadiiValue, BorderValue, ShadowValue, TransformValue, SemanticRole, SemanticCurrent, SemanticLive, SemanticCheckedState, SemanticOrientation, SemanticPopup, SemanticSort, KeyboardActivation } from '@argui/host'
export namespace JSX {
  export type Element = import('react').ReactElement
  export interface IntrinsicAttributes { key?: import('react').Key }
  export interface ElementChildrenAttribute { children: {} }
  export interface IntrinsicElements {
    container: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      background?: string
      selectionFill?: string
      selectionColor?: string
      selectionRadius?: number
      gap?: number
      loopMs?: number
      loopPlaying?: boolean
      loopGap?: number
      padding?: InsetsValue
      directionScope?: "ltr" | "rtl"
      wrap?: boolean
      grow?: number
      shrink?: number
      alignItems?: "start" | "center" | "end" | "stretch"
      justifyContent?: "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridRows?: GridTracksValue
      gridColumns?: GridTracksValue
      gridRowStart?: number
      gridRowSpan?: number
      gridColumnStart?: number
      gridColumnSpan?: number
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      aspectRatio?: number
      flexBasis?: DimensionValue
      rowGap?: number
      columnGap?: number
      zIndex?: number
      containerScope?: string
      margin?: InsetsValue
      transform?: TransformValue
      containerRules?: readonly ContainerRuleValue[]
      alignSelf?: "start" | "center" | "end" | "stretch"
      justifyItems?: "start" | "center" | "end" | "stretch"
      justifySelf?: "start" | "center" | "end" | "stretch"
      alignContent?: "start" | "center" | "end" | "stretch" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridAutoFlow?: "row" | "column" | "rowDense" | "columnDense"
      clip?: boolean
      radii?: RadiiValue
      border?: BorderValue
      shadow?: ShadowValue
      transitionMs?: number
      transitionSpring?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    row: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      background?: string
      selectionFill?: string
      selectionColor?: string
      selectionRadius?: number
      gap?: number
      loopMs?: number
      loopPlaying?: boolean
      loopGap?: number
      padding?: InsetsValue
      directionScope?: "ltr" | "rtl"
      wrap?: boolean
      grow?: number
      shrink?: number
      alignItems?: "start" | "center" | "end" | "stretch"
      justifyContent?: "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridRows?: GridTracksValue
      gridColumns?: GridTracksValue
      gridRowStart?: number
      gridRowSpan?: number
      gridColumnStart?: number
      gridColumnSpan?: number
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      aspectRatio?: number
      flexBasis?: DimensionValue
      rowGap?: number
      columnGap?: number
      zIndex?: number
      containerScope?: string
      margin?: InsetsValue
      transform?: TransformValue
      containerRules?: readonly ContainerRuleValue[]
      alignSelf?: "start" | "center" | "end" | "stretch"
      justifyItems?: "start" | "center" | "end" | "stretch"
      justifySelf?: "start" | "center" | "end" | "stretch"
      alignContent?: "start" | "center" | "end" | "stretch" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridAutoFlow?: "row" | "column" | "rowDense" | "columnDense"
      clip?: boolean
      radii?: RadiiValue
      border?: BorderValue
      shadow?: ShadowValue
      transitionMs?: number
      transitionSpring?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    column: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      background?: string
      selectionFill?: string
      selectionColor?: string
      selectionRadius?: number
      gap?: number
      loopMs?: number
      loopPlaying?: boolean
      loopGap?: number
      padding?: InsetsValue
      directionScope?: "ltr" | "rtl"
      wrap?: boolean
      grow?: number
      shrink?: number
      alignItems?: "start" | "center" | "end" | "stretch"
      justifyContent?: "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridRows?: GridTracksValue
      gridColumns?: GridTracksValue
      gridRowStart?: number
      gridRowSpan?: number
      gridColumnStart?: number
      gridColumnSpan?: number
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      aspectRatio?: number
      flexBasis?: DimensionValue
      rowGap?: number
      columnGap?: number
      zIndex?: number
      containerScope?: string
      margin?: InsetsValue
      transform?: TransformValue
      containerRules?: readonly ContainerRuleValue[]
      alignSelf?: "start" | "center" | "end" | "stretch"
      justifyItems?: "start" | "center" | "end" | "stretch"
      justifySelf?: "start" | "center" | "end" | "stretch"
      alignContent?: "start" | "center" | "end" | "stretch" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridAutoFlow?: "row" | "column" | "rowDense" | "columnDense"
      clip?: boolean
      radii?: RadiiValue
      border?: BorderValue
      shadow?: ShadowValue
      transitionMs?: number
      transitionSpring?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    text: ({
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      background?: string
      padding?: InsetsValue
      radius?: number
      loopMs?: number
      loopPlaying?: boolean
      loopOpacity?: number
      transitionMs?: number
      transitionSpring?: boolean
      selectionFill?: string
      selectionColor?: string
      selectionRadius?: number
      color?: string
      weight?: number
      fontSize?: number
      noWrap?: boolean
      lineHeight?: number
      fontStyle?: "normal" | "italic" | "oblique"
      letterSpacing?: number
      underline?: "none" | "single" | "double"
      strikethrough?: boolean
      textAlign?: "start" | "end" | "left" | "right" | "center" | "justify"
      lineClamp?: number
      textOverflow?: "clip" | "ellipsis" | "ellipsisStart" | "ellipsisMiddle" | "ellipsisEnd"
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
    }) & ({ text: string; children?: never } | { text?: never; children: string | number })
    image: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      alt?: string
      source: AssetRef
      fit?: "fill" | "contain" | "cover"
      sampling?: "linear" | "nearest"
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    svg: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      alt?: string
      source: AssetRef
      fit?: "fill" | "contain" | "cover"
      color?: string
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    rectangle: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      rotation?: number
      rotationLoopMs?: number
      loopMs?: number
      loopPlaying?: boolean
      loopTranslateX?: number
      loopTranslateY?: number
      loopScale?: number
      loopOpacity?: number
      loopBackground?: string
      loopHold?: boolean
      loopWidth?: number
      loopRadius?: number
      opacity?: number
      transitionMs?: number
      transitionSpring?: boolean
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      grow?: number
      shrink?: number
      alignSelf?: "start" | "center" | "end" | "stretch"
      margin?: InsetsValue
      padding?: InsetsValue
      background?: string
      hoverBackground?: string
      pressedBackground?: string
      pressedScale?: number
      border?: BorderValue
      radii?: RadiiValue
      shadow?: ShadowValue
      focusBorderColor?: string
      clip?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    touchArea: {
      id?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      enabled?: boolean
      mouseCursor?: "auto" | "default" | "contextMenu" | "help" | "pointer" | "progress" | "wait" | "cell" | "crosshair" | "text" | "verticalText" | "alias" | "copy" | "move" | "noDrop" | "notAllowed" | "grab" | "grabbing" | "eResize" | "nResize" | "neResize" | "nwResize" | "sResize" | "seResize" | "swResize" | "wResize" | "ewResize" | "nsResize" | "neswResize" | "nwseResize" | "colResize" | "rowResize" | "allScroll" | "zoomIn" | "zoomOut" | "dndAsk" | "allResize"
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onContextMenu?: (payload: NativeEventPayload<'contextMenu'>) => void
      onDoubleClicked?: (payload: NativeEventPayload<'click'>) => void
      onPointerEnter?: (payload: NativeEventPayload<'pointerEnter'>) => void
      onPointerLeave?: (payload: NativeEventPayload<'pointerLeave'>) => void
      onPointerDown?: (payload: NativeEventPayload<'pointerDown'>) => void
      onPointerUp?: (payload: NativeEventPayload<'pointerUp'>) => void
      onPointerMove?: (payload: NativeEventPayload<'pointerMove'>) => void
      onPointerCancel?: (payload: NativeEventPayload<'pointerCancel'>) => void
      onMoved?: (payload: NativeEventPayload<'pointerMove'>) => void
      onDragX?: (payload: NativeEventPayload<'gesture'>) => void
      onDragY?: (payload: NativeEventPayload<'gesture'>) => void
      onWheel?: (payload: NativeEventPayload<'wheel'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    focusScope: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      grow?: number
      shrink?: number
      alignSelf?: "start" | "center" | "end" | "stretch"
      margin?: InsetsValue
      enabled?: boolean
      focusOnClick?: boolean
      focusOnTabNavigation?: boolean
      containment?: "none" | "trap" | "modal"
      restoreFocus?: boolean
      initialFocus?: string
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      focusable?: boolean
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      selected?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      busy?: boolean
      checked?: boolean
      expandable?: boolean
      expanded?: boolean
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onKey?: (payload: NativeEventPayload<'key'>) => void
      onCaptureKey?: (payload: NativeEventPayload<'key'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    path: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      source: AssetRef
      fit?: "fill" | "contain" | "cover"
      color?: string
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    textInput: {
      id?: string
      value?: string
      placeholder?: string
      enabled?: boolean
      readOnly?: boolean
      search?: boolean
      multiline?: boolean
      privacy?: "public" | "password" | "revealedPassword"
      maxDigits?: number
      label?: string
      description?: string
      invalid?: boolean
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      background?: string
      clip?: boolean
      selectionFill?: string
      selectionRadius?: number
      textColor?: string
      placeholderColor?: string
      selectionColor?: string
      caretColor?: string
      caretFill?: string
      caretWidth?: number
      caretHeight?: number
      caretRadius?: number
      caretCount?: number
      caretSpacing?: number
      caretOffsetY?: number
      caretBlink?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      multiselectable?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onInput?: (payload: NativeEventPayload<'input'>) => void
      onEdit?: (payload: NativeEventPayload<'edit'>) => void
      onSubmit?: (payload: NativeEventPayload<'submit'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    keyBinding: {
      id?: string
      opacity?: number
      visible?: boolean
      shortcut: string
      enabled?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onActivated?: (payload: NativeEventPayload<'key'>) => void
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    popupWindow: {
      id?: string
      width?: DimensionValue
      height?: DimensionValue
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      anchor?: string
      placement?: "fill" | "center" | "topStart" | "top" | "topEnd" | "bottomStart" | "bottom" | "bottomEnd" | "leftStart" | "left" | "leftEnd" | "rightStart" | "right" | "rightEnd"
      placementOffset?: number
      dismissPolicy?: "manual" | "outsidePointer" | "escape" | "outsidePointerOrEscape" | "outsideHoverOrEscape"
      windowLayer?: "background" | "content" | "floating" | "popover" | "modal" | "debug"
      containment?: "none" | "trap" | "modal"
      restoreFocus?: boolean
      initialFocus?: string
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onDismiss?: (payload: NativeEventPayload<'dismiss'>) => void
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    scrollView: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      shrink?: number
      alignSelf?: "start" | "center" | "end" | "stretch"
      margin?: InsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      enabled?: boolean
      scrollX?: boolean
      scrollY?: boolean
      scrollbarSide?: "left" | "right"
      scrollbarWidth?: number
      scrollbarThumbColor?: string
      scrollbarHoverColor?: string
      grow?: number
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onScroll?: (payload: NativeEventPayload<'scroll'>) => void
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    virtualWindow: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      shrink?: number
      alignSelf?: "start" | "center" | "end" | "stretch"
      margin?: InsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      grow?: number
      rowHeight: number
      variableHeight?: boolean
      horizontal?: boolean
      viewportWidth?: number
      viewportHeight?: number
      offset?: number
      scrollbarThumb?: string
      scrollbarVisible?: boolean
      scrollbarWidth?: number
      shadowColor?: string
      shadowIntensity?: number
      shadowWidth?: number
      shadowStart?: boolean
      shadowEnd?: boolean
      overscan?: number
      itemCount?: number
      windowStart?: number
      dataVersion?: number
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onScroll?: (payload: NativeEventPayload<'scroll'>) => void
      onMeasure?: (payload: NativeEventPayload<'measure'>) => void
      onWindow?: (payload: NativeEventPayload<'window'>) => void
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    grid: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      minWidth?: ConstraintValue
      minHeight?: ConstraintValue
      background?: string
      selectionFill?: string
      selectionColor?: string
      selectionRadius?: number
      gap?: number
      loopMs?: number
      loopPlaying?: boolean
      loopGap?: number
      padding?: InsetsValue
      directionScope?: "ltr" | "rtl"
      wrap?: boolean
      grow?: number
      shrink?: number
      alignItems?: "start" | "center" | "end" | "stretch"
      justifyContent?: "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridRows?: GridTracksValue
      gridColumns?: GridTracksValue
      gridRowStart?: number
      gridRowSpan?: number
      gridColumnStart?: number
      gridColumnSpan?: number
      maxWidth?: ConstraintValue
      maxHeight?: ConstraintValue
      aspectRatio?: number
      flexBasis?: DimensionValue
      rowGap?: number
      columnGap?: number
      zIndex?: number
      containerScope?: string
      margin?: InsetsValue
      transform?: TransformValue
      containerRules?: readonly ContainerRuleValue[]
      alignSelf?: "start" | "center" | "end" | "stretch"
      justifyItems?: "start" | "center" | "end" | "stretch"
      justifySelf?: "start" | "center" | "end" | "stretch"
      alignContent?: "start" | "center" | "end" | "stretch" | "spaceBetween" | "spaceAround" | "spaceEvenly"
      gridAutoFlow?: "row" | "column" | "rowDense" | "columnDense"
      clip?: boolean
      radii?: RadiiValue
      border?: BorderValue
      shadow?: ShadowValue
      transitionMs?: number
      transitionSpring?: boolean
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onFocus?: (payload: NativeEventPayload<'focus'>) => void
      onBlur?: (payload: NativeEventPayload<'blur'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
    gpuCanvas: {
      id?: string
      tooltip?: string
      width?: DimensionValue
      height?: DimensionValue
      position?: "relative" | "absolute" | "sticky"
      inset?: PositionInsetsValue
      rotation?: number
      opacity?: number
      backdropFilter?: string
      desktopBackdropTint?: string
      desktopBackdropFallback?: string
      visible?: boolean
      canvasId: number
      revision?: number
      resolutionScale?: number
      sampling?: "linear" | "nearest"
      alt?: string
      role?: "generic" | "window" | "group" | "navigation" | "text" | "heading" | "image" | "link" | "button" | "checkBox" | "radioButton" | "radioGroup" | "switch" | "textInput" | "textArea" | "searchInput" | "table" | "grid" | "row" | "columnHeader" | "cell" | "list" | "listItem" | "tree" | "treeItem" | "listBox" | "option" | "menu" | "menuItem" | "menuBar" | "menuItemCheckBox" | "menuItemRadio" | "comboBox" | "tooltip" | "status" | "alertDialog" | "slider" | "progress" | "tab" | "tabList" | "tabPanel" | "dialog" | "alert" | "separator"
      accessibleName?: string
      accessibleDescription?: string
      accessibleValue?: string
      numericValue?: number
      minimumValue?: number
      maximumValue?: number
      valueStep?: number
      accessibleDisabled?: boolean
      accessibleHidden?: boolean
      focusable?: boolean
      focusOnTabNavigation?: boolean
      keyboardActivation?: "none" | "enter" | "enterOrSpace"
      selected?: boolean
      checked?: boolean
      checkedState?: "unchecked" | "checked" | "mixed"
      expanded?: boolean
      busy?: boolean
      current?: "true" | "page" | "step" | "location" | "date" | "time"
      pressedState?: boolean
      required?: boolean
      readOnly?: boolean
      multiselectable?: boolean
      invalid?: boolean
      live?: "off" | "polite" | "assertive"
      modal?: boolean
      orientation?: "horizontal" | "vertical"
      level?: number
      positionInSet?: number
      setSize?: number
      hasPopup?: "menu" | "listBox" | "tree" | "grid" | "dialog"
      sort?: "ascending" | "descending"
      controls?: string
      activeDescendant?: string
      labelledBy?: string
      describedBy?: string
      canIncrement?: boolean
      canDecrement?: boolean
      canSetValue?: boolean
      canExpand?: boolean
      canCollapse?: boolean
      canScrollIntoView?: boolean
      onClick?: (payload: NativeEventPayload<'click'>) => void
      onSemanticAction?: (payload: NativeEventPayload<'semanticAction'>) => void
      key?: import('react').Key
      ref?: Ref<NativeHandle>
      children?: ReactNode
    }
  }
}
