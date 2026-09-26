declare module '@argui/web-host/argui_app_web.js' {
  import type { NativeBridge, ThemeBridge } from '@argui/host'

  export default function init(): Promise<unknown>

  export class ArguiWebHost {
    constructor(parentId: string, assets: unknown[])
    contract: NativeBridge['contract']
    commit: NativeBridge['commit']
    subscribe: NativeBridge['subscribe']
    themeCreate: ThemeBridge['create']
    themeUpdate: ThemeBridge['update']
    themeSubscribe: ThemeBridge['subscribe']
    themeDispose: ThemeBridge['dispose']
  }
}
