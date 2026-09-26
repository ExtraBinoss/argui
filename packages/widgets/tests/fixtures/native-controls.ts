import { createThemeRuntime, NativeHost, type NativeBridge, type NativeDelivery, type NativeNode } from '@argui/host'
import contract from '../../../host/src/contract.generated.json' with { type: 'json' }
import { widgetThemeDefinition, type WidgetTheme } from '../../src/shared/theme'

/** Creates a native host and theme for exercising widget mounts without an OS window. */
export function nativeControlsFixture() {
  let deliver: (event: NativeDelivery) => void = () => {}
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: () => {},
    subscribe: (callback) => { deliver = callback; return () => {} },
    theme: {
      create: () => ({ id: 1, snapshot: {
        revision: 0, variant: 'light', resolvedVariant: 'light', systemScheme: 'light',
        values: widgetThemeDefinition.variants!.light as WidgetTheme, tokenRevisions: {},
      } }),
      update: () => { throw new Error('not used') },
      subscribe: () => () => {},
      dispose: () => {},
    },
  }
  const host = new NativeHost(bridge, contract.abiHash)
  const runtime = createThemeRuntime<WidgetTheme>(bridge, widgetThemeDefinition)
  return { host, runtime, deliver: (event: NativeDelivery) => deliver(event) }
}

/** Collects semantic role names from the mounted native tree. */
export function mountedRoles(root: NativeNode): string[] {
  const roles: string[] = []
  const visit = (node: NativeNode) => {
    const property = node.type.properties.find((item) => item.name === 'role')
    const role = property ? node.values.get(property.id)?.value : undefined
    if (typeof role === 'string') roles.push(role)
    node.children.forEach(visit)
  }
  visit(root)
  return roles
}

/** Finds the first mounted native node with the supplied role. */
export function mountedRole(root: NativeNode, role: string): NativeNode | undefined {
  const property = root.type.properties.find((item) => item.name === 'role')
  if (property && root.values.get(property.id)?.value === role) return root
  for (const child of root.children) {
    const found = mountedRole(child, role)
    if (found) return found
  }
  return undefined
}
