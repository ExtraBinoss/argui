/** Renderer features exposed by the embedded native host to gallery examples. */

export interface NativeEffectDefinition {
  id: string
  source: string
  parameters: readonly { name: string; type: 'f32' }[]
}

export interface RendererProfileSample {
  cpuMs: number
  gpuMs: number | null
  damageMode: 'full' | 'seed' | 'partial' | 'reused'
  regions: number
  damagedPixels: number
  viewportPixels: number
  retainedBytes: number
}

interface RendererBridge {
  control(request: { kind: 'damageTracking' | 'profile'; enabled: boolean }): void
  subscribeProfile(callback: (sample: RendererProfileSample) => void): () => void
}

type NativeGlobal = typeof globalThis & {
  __arguiNativeEffects?: NativeEffectDefinition[]
  __arguiBridge?: RendererBridge
}

/** Registers WGSL imported by a TSX module before the native renderer starts. */
export function registerNativeEffect(definition: NativeEffectDefinition): void {
  const native = globalThis as NativeGlobal
  const definitions = native.__arguiNativeEffects ?? (native.__arguiNativeEffects = [])
  const existing = definitions.find((candidate) => candidate.id === definition.id)
  if (existing) {
    const sameParameters = existing.parameters.length === definition.parameters.length
      && existing.parameters.every((parameter, index) =>
        parameter.name === definition.parameters[index]?.name
        && parameter.type === definition.parameters[index]?.type)
    if (existing.source !== definition.source || !sameParameters) {
      throw new Error(`Conflicting native effect ${definition.id}`)
    }
    return
  }
  definitions.push({
    id: definition.id,
    source: definition.source,
    parameters: definition.parameters.map((parameter) => ({ ...parameter })),
  })
}

/** Applies the engine's adaptive or full-frame damage policy. */
export function setNativeDamageTracking(enabled: boolean): boolean {
  const bridge = (globalThis as NativeGlobal).__arguiBridge
  if (!bridge?.control) return false
  bridge.control({ kind: 'damageTracking', enabled })
  return true
}

/** Subscribes to actual renderer samples while a visible example is active. */
export function subscribeRendererProfiles(
  callback: (sample: RendererProfileSample) => void,
): (() => void) | null {
  const bridge = (globalThis as NativeGlobal).__arguiBridge
  if (!bridge?.control || !bridge.subscribeProfile) return null
  const unsubscribe = bridge.subscribeProfile(callback)
  bridge.control({ kind: 'profile', enabled: true })
  return () => {
    unsubscribe()
    bridge.control({ kind: 'profile', enabled: false })
  }
}
