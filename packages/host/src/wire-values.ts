import type { AssetRef, NativeProperty, WireValue } from './protocol'

/** Encodes a schema value for the Rust bridge's tagged `SchemaValue` decoder. */
export function encodeValue(property: NativeProperty, value: unknown): WireValue {
  switch (property.valueType) {
    case 'Bool':
      if (typeof value !== 'boolean') break
      return { type: 'Bool', value }
    case 'Int':
      if (typeof value !== 'number' || !Number.isSafeInteger(value)) break
      return { type: 'Int', value }
    case 'Float':
      if (typeof value !== 'number' || !Number.isFinite(value)) break
      return { type: property.valueType, value }
    case 'String':
    case 'Name':
    case 'Color':
    case 'Brush':
      if (typeof value !== 'string') break
      if (property.allowedValues?.length && !property.allowedValues.includes(value)) break
      return { type: property.valueType, value }
    case 'Dimension':
      if (validDimension(value)) return { type: 'Dimension', value }
      break
    case 'Constraint':
      if (validDimension(value)) return { type: 'Constraint', value }
      break
    case 'GridTracks':
      if (Array.isArray(value) && value.every(validGridTrack)) return { type: 'GridTracks', value }
      break
    case 'ContainerRules':
      if (Array.isArray(value) && value.every(validContainerRule)) return { type: 'ContainerRules', value }
      break
    case 'Insets':
    case 'PositionInsets':
      if (validNumber(value) || (recordOfNumbers(value, ['top', 'right', 'bottom', 'left', 'start', 'end'])
        && !(('start' in value || 'end' in value) && ('left' in value || 'right' in value)))) {
        return { type: property.valueType, value }
      }
      break
    case 'Radii':
      if (validNumber(value) || recordOfNumbers(value, ['topLeft', 'topRight', 'bottomRight', 'bottomLeft'])) {
        return { type: 'Radii', value }
      }
      break
    case 'Border':
      if (plainObject(value) && Object.keys(value).length === 2 && typeof value.color === 'string'
        && (validNumber(value.width) || completeNumberRecord(value.width, ['top', 'right', 'bottom', 'left']))) {
        return { type: 'Border', value }
      }
      break
    case 'Shadow':
      if (plainObject(value) && typeof value.color === 'string' && validNumber(value.blur)
        && recordFields(value, ['offsetX', 'offsetY', 'blur', 'spread', 'color', 'inset'])
        && optionalNumber(value, 'offsetX') && optionalNumber(value, 'offsetY')
        && optionalNumber(value, 'spread') && (value.inset === undefined || typeof value.inset === 'boolean')) {
        return { type: 'Shadow', value }
      }
      break
    case 'Transform':
      if (recordOfNumbers(value, ['translateX', 'translateY', 'scaleX', 'scaleY', 'rotation'])) {
        return { type: 'Transform', value }
      }
      break
    case 'Asset':
      if (typeof value !== 'object' || !value) break
      {
        const asset = value as AssetRef
        if ((asset.kind !== 'image' && asset.kind !== 'svg') || !Number.isSafeInteger(asset.id) || asset.id <= 0) break
        return { type: 'Asset', value: { kind: asset.kind, id: asset.id } }
      }
  }
  throw new TypeError(`Invalid ${property.valueType} value for ${property.name}`)
}

function validNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function validDimension(value: unknown): value is number | string {
  return validNumber(value) || value === 'auto'
    || typeof value === 'string' && /^-?(?:\d+(?:\.\d*)?|\.\d+)%$/.test(value)
}

function plainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    && (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null)
}

function recordFields(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).every((key) => keys.includes(key))
}

function optionalNumber(value: Record<string, unknown>, key: string): boolean {
  return value[key] === undefined || validNumber(value[key])
}

function recordOfNumbers(value: unknown, keys: readonly string[]): value is Record<string, number> {
  return plainObject(value) && recordFields(value, keys)
    && Object.keys(value).every((key) => validNumber(value[key]))
}

function completeNumberRecord(value: unknown, keys: readonly string[]): boolean {
  return recordOfNumbers(value, keys) && keys.every((key) => key in value)
}

function validGridTrackSingle(value: unknown): boolean {
  if (validNumber(value) || value === 'auto') return true
  if (typeof value === 'string') return /^-?(?:\d+(?:\.\d*)?|\.\d+)%$/.test(value)
  if (!plainObject(value)) return false
  if (Object.keys(value).length !== 1) return false
  if ('fr' in value) return validNumber(value.fr) && value.fr > 0
  if (!plainObject(value.minmax) || !recordFields(value.minmax, ['min', 'max'])
    || Object.keys(value.minmax).length !== 2 || !validNumber(value.minmax.min)) return false
  const max = value.minmax.max
  return validNumber(max) || plainObject(max) && Object.keys(max).length === 1
    && validNumber(max.fr) && max.fr > 0
}

function validGridTrack(value: unknown): boolean {
  if (validGridTrackSingle(value)) return true
  if (!plainObject(value) || Object.keys(value).length !== 1 || !plainObject(value.repeat)
    || !recordFields(value.repeat, ['count', 'tracks']) || Object.keys(value.repeat).length !== 2) return false
  const { count, tracks } = value.repeat
  return (count === 'autoFit' || count === 'autoFill' || Number.isSafeInteger(count) && (count as number) > 0)
    && Array.isArray(tracks) && tracks.length > 0 && tracks.every(validGridTrackSingle)
}

function validContainerRule(value: unknown): boolean {
  if (!plainObject(value) || !recordFields(value, ['scope', 'when', 'style'])
    || Object.keys(value).length !== 3 || typeof value.scope !== 'string' || !value.scope
    || !plainObject(value.when) || !plainObject(value.style)) return false
  const when = value.when
  const style = value.style
  if (!Object.keys(when).length || !Object.keys(style).length
    || !recordFields(when, ['minWidth', 'maxWidth', 'minHeight', 'maxHeight', 'orientation'])
    || !recordFields(style, ['gridColumns', 'gridRows', 'width', 'height', 'gap', 'grow', 'shrink', 'alignItems', 'justifyContent'])) return false
  for (const name of ['minWidth', 'maxWidth', 'minHeight', 'maxHeight']) {
    if (when[name] !== undefined && (!validNumber(when[name]) || (when[name] as number) < 0)) return false
  }
  if (when.orientation !== undefined && when.orientation !== 'landscape' && when.orientation !== 'portrait') return false
  for (const name of ['gridColumns', 'gridRows']) {
    const tracks = style[name]
    if (tracks !== undefined && (!Array.isArray(tracks) || !tracks.every(validGridTrack))) return false
  }
  for (const name of ['width', 'height']) {
    if (style[name] !== undefined && !validDimension(style[name])) return false
  }
  for (const name of ['gap', 'grow', 'shrink']) {
    if (style[name] !== undefined && !validNumber(style[name])) return false
  }
  if (style.alignItems !== undefined && !['start', 'center', 'end', 'stretch'].includes(style.alignItems as string)) return false
  if (style.justifyContent !== undefined && !['start', 'center', 'end', 'spaceBetween', 'spaceAround', 'spaceEvenly'].includes(style.justifyContent as string)) return false
  return true
}

/** Compares wire values, including stable asset identity, to avoid redundant native mutations. */
export function equalValue(left: WireValue | undefined, right: WireValue): boolean {
  if (left?.type !== right.type) return false
  if (right.type === 'Asset') {
    const previous = left.value as AssetRef
    const next = right.value as AssetRef
    return previous.kind === next.kind && previous.id === next.id
  }
  if (right.type === 'GridTracks' || right.type === 'ContainerRules' || right.type === 'Insets' || right.type === 'PositionInsets'
    || right.type === 'Radii' || right.type === 'Border'
    || right.type === 'Shadow' || right.type === 'Transform') {
    return JSON.stringify(left.value) === JSON.stringify(right.value)
  }
  return left.value === right.value
}
