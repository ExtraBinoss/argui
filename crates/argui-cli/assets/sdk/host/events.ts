import type { SemanticActionName } from './accessibility'

/** Pointer geometry added by native delivery when it is available. */
export interface NativePointerGeometry {
  x?: number
  y?: number
  localX?: number
  localY?: number
  width?: number
  height?: number
}

/** Public payload shapes indexed by the schema's native event type. */
export interface NativeEventPayloads {
  action: { kind: 'action' }
  pointerEnter: { kind: 'pointerEnter' } & NativePointerGeometry
  pointerLeave: { kind: 'pointerLeave' } & NativePointerGeometry
  pointerMove: { kind: 'pointerMove' } & NativePointerGeometry
  pointerDown: { kind: 'pointerDown' } & NativePointerGeometry
  pointerOutside: { kind: 'pointerOutside' } & NativePointerGeometry
  dismiss: { kind: 'dismiss' }
  pointerUp: { kind: 'pointerUp' } & NativePointerGeometry
  pointerCancel: { kind: 'pointerCancel' } & NativePointerGeometry
  click: { kind: 'click' } & NativePointerGeometry
  contextMenu: { kind: 'contextMenu' } & NativePointerGeometry
  gotPointerCapture: { kind: 'gotPointerCapture' }
  lostPointerCapture: { kind: 'lostPointerCapture' }
  key: {
    kind: 'key'; key: string; state: 'pressed' | 'released'; text: string | null
    shift: boolean; control: boolean; alt: boolean; super: boolean; repeat: boolean
  } & NativePointerGeometry
  wheel: { kind: 'wheel' } & NativePointerGeometry
  scroll: { kind: 'scroll'; offsetX: number; offsetY: number } & NativePointerGeometry
  measure: {
    kind: 'measure'; items: readonly { index: number; extent: number }[]
    correctedOffset: number; viewportExtent: number
  } & NativePointerGeometry
  window: {
    kind: 'window'; start: number; end: number; offset: number; viewportExtent: number
  } & NativePointerGeometry
  focus: { kind: 'focus' }
  blur: { kind: 'blur' }
  input: { kind: 'input'; text: string }
  edit: { kind: 'edit'; start: number; end: number; text: string }
  submit: { kind: 'submit'; text: string }
  gesture: { kind: 'gesture' } | ({
    kind: 'pan'; deltaX: number; deltaY: number; totalX: number; totalY: number
    velocityX: number; velocityY: number; phase: 'started' | 'changed' | 'ended' | 'cancelled'
  } & NativePointerGeometry)
  semanticAction: { kind: 'semanticAction'; action: SemanticActionName; value: string | number | null }
  selectionChange: { kind: 'selectionChange' }
}

/** Payload delivered for one event type declared in the Rust schema. */
export type NativeEventPayload<T extends keyof NativeEventPayloads> = NativeEventPayloads[T]
