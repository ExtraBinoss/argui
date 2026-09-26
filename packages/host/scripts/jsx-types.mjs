const valueTypes = {
  Bool: 'boolean', Int: 'number', Float: 'number', String: 'string', Name: 'string',
  Color: 'string', Brush: 'string', Dimension: 'DimensionValue', Constraint: 'ConstraintValue',
  Insets: 'InsetsValue', PositionInsets: 'PositionInsetsValue', Radii: 'RadiiValue', Border: 'BorderValue',
  Shadow: 'ShadowValue', Transform: 'TransformValue', Asset: 'AssetRef',
  GridTracks: 'GridTracksValue',
  ContainerRules: 'readonly ContainerRuleValue[]',
}

const semanticTypes = {
  role: 'SemanticRole', current: 'SemanticCurrent', live: 'SemanticLive',
  checkedState: 'SemanticCheckedState', keyboardActivation: 'KeyboardActivation',
  orientation: 'SemanticOrientation', hasPopup: 'SemanticPopup', sort: 'SemanticSort',
}

const contractName = /^[a-z][a-zA-Z0-9]*$/

function propertyType(property) {
  if (property.allowedValues?.length) {
    for (const option of property.allowedValues) {
      if (!contractName.test(option)) throw new Error(`Non-camelCase option ${property.name}=${option}`)
    }
    return property.allowedValues.map((option) => JSON.stringify(option)).join(' | ')
  }
  return semanticTypes[property.name] ?? valueTypes[property.valueType]
    ?? (() => { throw new Error(`Unsupported schema type ${property.valueType}`) })()
}

/** Generates framework JSX types directly from the Rust schema contract. */
export function generateJSX(contract, framework, augmentation = false) {
  if (framework !== 'react' && framework !== 'solid') throw new Error(`Unsupported JSX framework ${framework}`)
  const react = framework === 'react'
  const imports = react
    ? ["import type { ReactNode, Ref } from 'react'", "import type { NativeHandle, NativeEventPayload, AssetRef, DimensionValue, ConstraintValue, GridTracksValue, ContainerRuleValue, InsetsValue, PositionInsetsValue, RadiiValue, BorderValue, ShadowValue, TransformValue, SemanticRole, SemanticCurrent, SemanticLive, SemanticCheckedState, SemanticOrientation, SemanticPopup, SemanticSort, KeyboardActivation } from '@argui/host'"]
    : ["import type { NativeHandle, NativeNode, NativeEventPayload, AssetRef, DimensionValue, ConstraintValue, GridTracksValue, ContainerRuleValue, InsetsValue, PositionInsetsValue, RadiiValue, BorderValue, ShadowValue, TransformValue, SemanticRole, SemanticCurrent, SemanticLive, SemanticCheckedState, SemanticOrientation, SemanticPopup, SemanticSort, KeyboardActivation } from '@argui/host'"]
  const lines = [
    `// Generated from argui-schema by packages/${framework}/scripts/generate-jsx.mjs.`,
    ...imports,
    ...(augmentation ? [`declare module '@argui/${framework}/jsx-runtime' {`, '  export namespace JSX {'] : ['export namespace JSX {']),
    ...(!augmentation ? [
      react ? "  export type Element = import('react').ReactElement" : "  export type Element = import('solid-js').JSX.Element",
      react ? "  export interface IntrinsicAttributes { key?: import('react').Key }" : '  export interface IntrinsicAttributes { key?: string | number }',
      '  export interface ElementChildrenAttribute { children: {} }',
    ] : []),
    '  export interface IntrinsicElements {',
  ]
  for (const native of contract.natives) {
    const tag = native.name[0].toLowerCase() + native.name.slice(1)
    const props = []
    for (const property of native.properties) {
      if (property.readOnly) continue
      if (!contractName.test(property.name)) throw new Error(`Non-camelCase property ${native.name}.${property.name}`)
      props.push(`      ${property.name}${property.required ? ':' : '?:'} ${propertyType(property)}`)
    }
    for (const event of native.events) {
      if (!contractName.test(event.name)) throw new Error(`Non-camelCase event ${native.name}.${event.name}`)
      if (!event.eventType || !contractName.test(event.eventType[0].toLowerCase() + event.eventType.slice(1))) {
        throw new Error(`Missing event type for ${native.name}.${event.name}`)
      }
      const callback = `on${event.name[0].toUpperCase()}${event.name.slice(1)}`
      props.push(`      ${callback}?: (payload: NativeEventPayload<'${event.eventType}'>) => void`)
    }
    props.push(react ? "      key?: import('react').Key" : '      key?: string | number')
    props.push(react ? '      ref?: Ref<NativeHandle>' : '      ref?: (node: NativeHandle) => void')
    if (tag === 'text') {
      const textIndex = props.findIndex((line) => /^\s+text[?:]/.test(line))
      if (textIndex < 0) throw new Error('Text primitive lacks text property')
      props.splice(textIndex, 1)
      const body = props.join('\n')
      lines.push(`    ${tag}: ({\n${body}\n    }) & ({ text: string; children?: never } | { text?: never; children: string | number })`)
    } else {
      props.push(react ? '      children?: ReactNode' : '      children?: Element | readonly Element[]')
      lines.push(`    ${tag}: {\n${props.join('\n')}\n    }`)
    }
  }
  lines.push('  }', '}', ...(augmentation ? ['}'] : []), '')
  return lines.join('\n')
}
