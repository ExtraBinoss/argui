/** Native roles shared by Solid, React, AccessKit and the web semantic adapter. */
export type SemanticRole =
  | 'generic' | 'window' | 'group' | 'navigation' | 'text' | 'heading' | 'image' | 'link'
  | 'button' | 'check_box' | 'radio_button' | 'switch' | 'text_input' | 'text_area'
  | 'search_input' | 'table' | 'grid' | 'row' | 'column_header' | 'cell' | 'list'
  | 'list_item' | 'tree' | 'tree_item' | 'list_box' | 'option' | 'menu' | 'menu_item'
  | 'menu_bar' | 'menu_item_check_box' | 'menu_item_radio' | 'combo_box' | 'tooltip'
  | 'status' | 'alert_dialog' | 'slider' | 'progress' | 'tab' | 'tab_list'
  | 'tab_panel' | 'dialog' | 'alert' | 'separator'

/** Relationship of an item to its current set or location. */
export type SemanticCurrent = 'true' | 'page' | 'step' | 'location' | 'date' | 'time'

/** Priority used for accessible live announcements. */
export type SemanticLive = 'off' | 'polite' | 'assertive'

/** Persistent checked state, including a partially checked control. */
export type SemanticCheckedState = 'unchecked' | 'checked' | 'mixed'

/** Direction of a semantic list, slider, separator or similar component. */
export type SemanticOrientation = 'horizontal' | 'vertical'

/** Kind of popup announced for a controlling element. */
export type SemanticPopup = 'menu' | 'list_box' | 'tree' | 'grid' | 'dialog'

/** Sort direction announced for a table column. */
export type SemanticSort = 'ascending' | 'descending'

/** Keyboard keys that activate an authored custom control. */
export type KeyboardActivation = 'none' | 'enter' | 'enter_or_space'

/** Action delivered from the platform accessibility adapter to a custom control. */
export type SemanticActionName = 'click' | 'focus' | 'blur' | 'increment' | 'decrement'
  | 'expand' | 'collapse' | 'set_value' | 'scroll_into_view'

/** Payload delivered to an authored `onSemanticAction` callback. */
export interface SemanticActionPayload {
  kind: 'semantic_action'
  action: SemanticActionName
  value: string | number | null
}
