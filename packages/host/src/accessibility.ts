/** Native roles shared by Solid, React, AccessKit and the web semantic adapter. */
export type SemanticRole =
  | 'generic' | 'window' | 'group' | 'navigation' | 'text' | 'heading' | 'image' | 'link'
  | 'button' | 'checkBox' | 'radioButton' | 'radioGroup' | 'switch' | 'textInput' | 'textArea'
  | 'searchInput' | 'table' | 'grid' | 'row' | 'columnHeader' | 'cell' | 'list'
  | 'listItem' | 'tree' | 'treeItem' | 'listBox' | 'option' | 'menu' | 'menuItem'
  | 'menuBar' | 'menuItemCheckBox' | 'menuItemRadio' | 'comboBox' | 'tooltip'
  | 'status' | 'alertDialog' | 'slider' | 'progress' | 'tab' | 'tabList'
  | 'tabPanel' | 'dialog' | 'alert' | 'separator'

/** Relationship of an item to its current set or location. */
export type SemanticCurrent = 'true' | 'page' | 'step' | 'location' | 'date' | 'time'

/** Priority used for accessible live announcements. */
export type SemanticLive = 'off' | 'polite' | 'assertive'

/** Persistent checked state, including a partially checked control. */
export type SemanticCheckedState = 'unchecked' | 'checked' | 'mixed'

/** Direction of a semantic list, slider, separator or similar component. */
export type SemanticOrientation = 'horizontal' | 'vertical'

/** Kind of popup announced for a controlling element. */
export type SemanticPopup = 'menu' | 'listBox' | 'tree' | 'grid' | 'dialog'

/** Sort direction announced for a table column. */
export type SemanticSort = 'ascending' | 'descending'

/** Keyboard keys that activate an authored custom control. */
export type KeyboardActivation = 'none' | 'enter' | 'enterOrSpace'

/** Action delivered from the platform accessibility adapter to a custom control. */
export type SemanticActionName = 'click' | 'focus' | 'blur' | 'increment' | 'decrement'
  | 'expand' | 'collapse' | 'setValue' | 'scrollIntoView'

/** Payload delivered to an authored `onSemanticAction` callback. */
export interface SemanticActionPayload {
  kind: 'semanticAction'
  action: SemanticActionName
  value: string | number | null
}
