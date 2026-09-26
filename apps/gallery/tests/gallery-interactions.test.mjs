import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { mountGallery as mountSolid } from '../dist/gallery-core.mjs'
import { mountGallery as mountReact } from '../dist/gallery-react-core.mjs'
import { createThemeBridge } from './fixtures/theme-bridge.mjs'

const contract = JSON.parse(readFileSync('packages/host/src/contract.generated.json', 'utf8'))
const typesById = new Map(contract.natives.map((type) => [type.id, type]))
const propertiesById = new Map(contract.natives.flatMap((type) => type.properties.map((property) => [property.id, property])))

function key(id) { return `${id.slot}:${id.generation}` }

function galleryBridge() {
  const nodes = new Map()
  const commits = []
  let root = null
  let deliver = () => {}
  const bridge = {
    contract: () => contract,
    subscribe(callback) { deliver = callback; return () => { deliver = () => {} } },
    theme: createThemeBridge(),
    commit(operations) {
      commits.push(operations.map(({ kind, nativeType, property }) => ({ kind, nativeType, property })))
      for (const operation of operations) {
        if (operation.kind === 'create') {
          nodes.set(key(operation.id), {
            id: operation.id,
            type: typesById.get(operation.nativeType),
            properties: {},
            listeners: new Map(),
            children: [],
            parent: null,
          })
        } else if (operation.kind === 'setProperty') {
          const node = nodes.get(key(operation.id))
          const property = propertiesById.get(operation.property)
          if (node && property) {
            if (operation.value === null) delete node.properties[property.name]
            else node.properties[property.name] = operation.value.value
          }
        } else if (operation.kind === 'setListener') {
          const node = nodes.get(key(operation.id))
          const event = node?.type.events.find((candidate) => candidate.id === operation.event)
          if (node && event) {
            if (operation.callback === null) node.listeners.delete(event.name)
            else node.listeners.set(event.name, operation.callback)
          }
        } else if (operation.kind === 'insert') {
          const parent = nodes.get(key(operation.parent))
          const child = nodes.get(key(operation.child))
          if (!parent || !child) continue
          if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1)
          child.parent = parent
          const index = operation.before
            ? parent.children.findIndex((entry) => key(entry.id) === key(operation.before))
            : -1
          parent.children.splice(index < 0 ? parent.children.length : index, 0, child)
        } else if (operation.kind === 'remove') {
          const node = nodes.get(key(operation.id))
          if (node?.parent) node.parent.children.splice(node.parent.children.indexOf(node), 1)
          const removeTree = (entry) => {
            for (const child of entry.children) removeTree(child)
            nodes.delete(key(entry.id))
          }
          if (node) removeTree(node)
          if (root && key(root.id) === key(operation.id)) root = null
        } else if (operation.kind === 'setRoot') {
          root = operation.id ? nodes.get(key(operation.id)) ?? null : null
        }
      }
    },
  }
  return {
    bridge,
    commits,
    find(typeName, predicate = () => true) {
      const visit = (node) => {
        if (!node) return undefined
        if (node.type.name === typeName && predicate(node)) return node
        for (const child of node.children) {
          const match = visit(child)
          if (match) return match
        }
        return undefined
      }
      return visit(root)
    },
    all(typeName) {
      const result = []
      const visit = (node) => {
        if (!node) return
        if (node.type.name === typeName) result.push(node)
        for (const child of node.children) visit(child)
      }
      visit(root)
      return result
    },
    text(node) {
      if (!node) return ''
      const own = typeof node.properties.text === 'string' ? node.properties.text : ''
      return own + node.children.map((child) => this.text(child)).join('')
    },
    dispatch(node, eventName, payload = {}) {
      const callback = node?.listeners.get(eventName)
      assert(callback, `${node?.type.name ?? 'missing node'} has no ${eventName} listener`)
      deliver({ node: node.id, callback, payload })
    },
  }
}

const turn = () => new Promise((resolve) => setTimeout(resolve, 0))

function findButton(view, label) {
  return view.find('FocusScope', (node) => node.properties.role === 'button' && view.text(node) === label)
}

function descendant(node, typeName) {
  if (!node) return undefined
  if (node.type.name === typeName) return node
  for (const child of node.children) {
    const match = descendant(child, typeName)
    if (match) return match
  }
  return undefined
}

for (const [adapter, mount] of [['Solid', mountSolid], ['React', mountReact]]) {
  test(`${adapter} gallery exposes matching layout geometry and settles after theme updates`, async (context) => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      let selected = view.find('FocusScope', (node) => node.properties.id === 'page-button')
      let surface = descendant(selected, 'Rectangle')
      assert.equal(surface?.properties.background, 'oklch(0.97 0 0)', 'light selected navigation uses sidebar accent')
      assert.equal(descendant(selected, 'Text')?.properties.textColor, 'oklch(0.205 0 0)', 'selected navigation label uses sidebar primary')
      assert.equal(surface?.properties.hoverBackground, 'oklch(0.97 0 0)', 'ghost hover follows the official muted role')
      assert.equal(selected?.properties.pressBounceScale, 0.97, 'native press bounces the entire control')

      view.dispatch(findButton(view, 'Layouting'), 'click')
      await turn()
      await turn()
      await turn()
      for (const id of [
        'layout-fixed-grow-320', 'layout-fixed-grow-640', 'layout-long-text-row',
        'layout-bounded-scroll', 'layout-auto-fit-grid-320', 'layout-auto-fit-grid-640',
        'layout-rtl-inset',
      ]) {
        assert(view.find('Row', (node) => node.properties.id === id)
          || view.find('ScrollView', (node) => node.properties.id === id)
          || view.find('Grid', (node) => node.properties.id === id)
          || view.find('Container', (node) => node.properties.id === id), `${id} is mounted`)
      }
      assert(view.all('Rectangle').some((node) => node.properties.id === 'layout-rtl-marker'),
        'logical inset marker is mounted')
      const idleStart = view.commits.length
      await turn()
      await turn()
      const idleCommits = view.commits.length - idleStart
      assert.equal(idleCommits, 0, 'settled gallery emits no idle native transactions')

      view.dispatch(findButton(view, 'Settings'), 'click')
      await turn()
      const themeStart = view.commits.length
      view.dispatch(findButton(view, 'Dark'), 'click')
      await turn()
      await turn()
      const themeBatches = view.commits.slice(themeStart).flat()
      assert(themeBatches.length > 0, 'theme update emits native properties')
      assert(themeBatches.every((operation) => operation.kind === 'setProperty'),
        `theme update changes existing native properties without rebuilding nodes: ${JSON.stringify(themeBatches.filter((operation) => operation.kind !== 'setProperty'))}`)
      selected = view.find('FocusScope', (node) => node.properties.id === 'page-layouting')
      surface = descendant(selected, 'Rectangle')
      assert.equal(surface?.properties.background, 'oklch(0.269 0 0)', 'dark selected navigation uses sidebar accent')
      assert.equal(surface?.properties.hoverBackground, 'oklch(0.269 0 0)', 'sidebar hover uses its own accent role')
      assert.equal(descendant(selected, 'Text')?.properties.textColor, 'oklch(0.488 0.243 264.376)', 'dark selected label uses sidebar primary')
      assert.equal(view.find('Text', (node) => node.properties.id === 'layout-long-text')?.properties.textColor,
        'oklch(0.985 0 0)', 'example text follows the dark text token')
      view.dispatch(findButton(view, 'System'), 'click')
      await turn()
      assert.equal(view.find('Column', (node) => node.properties.id === 'gallery-sidebar')?.properties.background,
        'oklch(0.985 0 0)', 'System resolves to the fixture\'s light desktop scheme')
      assert.equal(findButton(view, 'System')?.properties.pressedState, true, 'System is a distinct selected choice')
      context.diagnostic(`idle transactions=${idleCommits}; theme transactions=${view.commits.length - themeStart}; theme operations=${themeBatches.length}`)
    } finally { dispose() }
  })

  test(`${adapter} Button dispatch updates rendered state`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'InputField'), 'click')
      await turn()
      view.dispatch(findButton(view, 'Button'), 'click')
      await turn()
      view.dispatch(findButton(view, 'Default'), 'click')
      await turn()
      assert(view.all('Text').some((node) => node.properties.text === 'Clicked 1 times'))
      assert.equal(findButton(view, 'Default')?.properties.accessibleName, 'Default')
      const disabled = findButton(view, 'Disabled')
      assert.equal(disabled?.properties.mouseCursor, 'notAllowed')
      assert.equal(disabled?.properties.enabled, false)
      assert.equal(disabled?.children[0]?.type.name, 'Rectangle', 'button press uses its focus scope without an extra hit region')
      const opened = []
      const previousWindow = globalThis.window
      globalThis.window = { location: { assign: (...args) => opened.push(args) } }
      try {
        view.dispatch(findButton(view, 'Link'), 'click')
        await turn()
      } finally {
        if (previousWindow === undefined) delete globalThis.window
        else globalThis.window = previousWindow
      }
      assert.deepEqual(opened, [['https://extrabinoss.github.io/argui/']])
    } finally { dispose() }
  })

  test(`${adapter} color family and appearance update shared button paint`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'Settings'), 'click')
      await turn()
      const colors = view.find('ScrollView', (node) => node.properties.id === 'gallery-color-list')
      assert.equal(colors?.properties.scrollY, true)
      assert.equal(colors?.properties.height, 228)
      assert.equal(colors?.children[0]?.type.name, 'Column', 'color choices form a vertical overflow list')
      assert(view.all('FocusScope').filter((node) => node.properties.id?.startsWith('color-')).length > 15,
        'the bounded viewport contains more choices than one screen can show')
      view.dispatch(findButton(view, 'Blue'), 'click')
      await turn()
      assert.equal(descendant(findButton(view, 'Default'), 'Rectangle')?.properties.background,
        'oklch(0.488 0.243 264.376)')
      assert.equal(findButton(view, 'Blue')?.properties.pressedState, true)
      view.dispatch(findButton(view, 'Dark'), 'click')
      await turn()
      assert.equal(descendant(findButton(view, 'Default'), 'Rectangle')?.properties.background,
        'oklch(0.707 0.165 254.624)')
      view.dispatch(findButton(view, 'System'), 'click')
      await turn()
      assert.equal(descendant(findButton(view, 'Default'), 'Rectangle')?.properties.background,
        'oklch(0.488 0.243 264.376)', 'system mode follows the light fixture scheme')
      view.bridge.theme.update(1, { systemScheme: 'dark' })
      await turn()
      assert.equal(descendant(findButton(view, 'Default'), 'Rectangle')?.properties.background,
        'oklch(0.707 0.165 254.624)', 'system changes refresh the chosen family')
      view.dispatch(findButton(view, 'Neutral'), 'click')
      await turn()
      assert.equal(descendant(findButton(view, 'Default'), 'Rectangle')?.properties.background,
        'oklch(0.922 0 0)', 'Neutral restores the official dark primary')
    } finally { dispose() }
  })

  test(`${adapter} motion toggle changes the native press animation`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      const demo = () => findButton(view, 'Click repeatedly')
      assert.equal(descendant(findButton(view, 'Rounded'), 'Rectangle')?.properties.radii, 16)
      assert.equal(descendant(findButton(view, 'Small'), 'Rectangle')?.properties.radii, 8)
      assert.equal(demo()?.properties.pressBounceScale, 0.97)
      assert.equal(descendant(demo(), 'Rectangle')?.properties.transitionMs, 150)
      assert.equal(descendant(demo(), 'Rectangle')?.properties.transitionTimingFunction, 'cubic-bezier(0.4, 0, 0.2, 1)')
      view.dispatch(findButton(view, 'Motion: On'), 'click')
      await turn()
      assert(findButton(view, 'Motion: Off'))
      assert.equal(demo()?.properties.pressBounceScale, undefined)
      view.dispatch(findButton(view, 'Motion: Off'), 'click')
      await turn()
      assert.equal(demo()?.properties.pressBounceScale, 0.97)
    } finally { dispose() }
  })

  test(`${adapter} ButtonGroup composes actions, search, vertical flow, and RTL`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'ButtonGroup'), 'click')
      await turn()
      const actions = view.find('Rectangle', (node) => node.properties.id === 'group-actions')
      assert.equal(actions?.properties.role, 'group')
      assert.equal(actions?.properties.accessibleName, 'Document actions')
      assert.equal(actions?.properties.clip, true, 'joined controls share a clipped outer border')
      view.dispatch(findButton(view, 'Archive'), 'click')
      await turn()
      assert(view.all('Text').some((node) => node.properties.text === 'Archived'))

      const searchInput = view.find('TextInput', (node) => node.properties.label === 'Search term')
      assert(searchInput, 'grouped search editor is mounted')
      view.dispatch(searchInput, 'edit', { kind: 'edit', start: 0, end: 0, text: 'snoo' })
      await turn()
      view.dispatch(findButton(view, 'Search'), 'click')
      await turn()
      assert(view.all('Text').some((node) => node.properties.text === 'Matches: Snooze'))

      const vertical = view.find('Rectangle', (node) => node.properties.id === 'group-vertical')
      assert.equal(vertical?.properties.role, 'group')
      assert.equal(descendant(view.find('Rectangle', (node) => node.properties.id === 'group-rtl'), 'Row')?.properties.directionScope, 'rtl')
      view.dispatch(findButton(view, 'More'), 'click')
      await turn()
      const schedule = findButton(view, 'Schedule')
      assert(schedule, 'split button popover can mount its own controls')
      assert.equal(descendant(schedule, 'Rectangle')?.properties.radii, 10,
        'floating controls regain their own corners outside the trigger group')
    } finally { dispose() }
  })

  test(`${adapter} examples remain distinct and native loops can be paused`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      assert(view.all('Text').some((node) => node.properties.text === 'Widgets'))
      assert(view.all('Text').some((node) => node.properties.text === 'Examples'))
      const navigation = view.find('ScrollView', (node) => node.properties.id === 'gallery-navigation')
      assert(navigation)
      const navigationFrame = view.find('Rectangle', (node) => node.properties.id === 'gallery-navigation-frame')
      assert.equal(navigationFrame?.properties.grow, 1, 'navigation fills the remaining sidebar height')
      assert.equal(navigationFrame?.properties.minHeight, 0)
      for (const side of ['top', 'bottom']) {
        const shade = view.find('Rectangle', (node) => node.properties.id === `gallery-nav-shade-${side}`)
        assert.equal(shade?.properties.height, 28)
        assert.match(shade?.properties.background.stops[0].color, /^oklch\([\d.]+ [\d.]+ [\d.]+ \/ \d+%\)$/,
          `${side} shadow keeps the sidebar hue and darkens it`)
      }
      assert.equal(descendant(navigation, 'Svg'), undefined, 'sidebar navigation does not mount icons')
      const inputNavigationId = view.find('FocusScope', (node) => node.properties.id === 'page-input-field')?.id
      view.dispatch(findButton(view, 'Layouting'), 'click')
      await turn()
      assert.deepEqual(view.find('FocusScope', (node) => node.properties.id === 'page-input-field')?.id,
        inputNavigationId, 'switching pages preserves native navigation identities')
      assert(view.find('Row', (node) => node.properties.id === 'layout-live-row'))
      view.dispatch(findButton(view, 'Expand to 520 px'), 'click')
      await turn()
      assert.equal(view.find('Row', (node) => node.properties.id === 'layout-live-row')?.properties.width, 520)
      view.dispatch(findButton(view, 'Animation'), 'click')
      await turn()
      assert.equal(view.find('Row', (node) => node.properties.id === 'layout-live-row'), undefined)
      for (const id of ['animation-travel', 'animation-steps', 'animation-scale', 'animation-color', 'animation-width']) {
        const scene = view.find('Rectangle', (node) => node.properties.id === id)
        assert(scene, `${id} is mounted`)
        assert(descendant(scene, 'Text'), `${id} includes explanatory text`)
        assert(scene.properties.loopMs > 0, `${id} uses native loop timing`)
      }
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'animation-steps')?.properties.loopSteps, 6)
      view.dispatch(findButton(view, 'Pause animations'), 'click')
      await turn()
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'animation-travel')?.properties.loopPlaying, false)
    } finally { dispose() }
  })

  test(`${adapter} expressive examples keep gradients and loops in native nodes`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'Expressive UI'), 'click')
      await turn()
      const gradient = view.find('Rectangle', (node) => node.properties.id === 'expressive-click-gradient')
      assert.equal(gradient?.properties.background.kind, 'linear')
      assert.equal(gradient?.properties.opacity, 0)
      for (const id of ['expressive-voice-cyan', 'expressive-dotted-spinner',
        'expressive-breathing-halo', 'expressive-sweep']) {
        assert(view.find('Rectangle', (node) => node.properties.id === id), `${id} is mounted`)
      }
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'expressive-voice-cyan')?.properties.background.kind, 'radial')
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'expressive-dotted-spinner')?.properties.rotationLoopMs, 1200)

      view.dispatch(view.find('FocusScope', (node) => node.properties.id === 'expressive-glow-button'), 'click')
      await turn()
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'expressive-click-gradient')?.properties.opacity, 0.95)
      view.dispatch(view.find('FocusScope', (node) => node.properties.id === 'expressive-record-button'), 'click')
      await turn()
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'expressive-voice-cyan')?.properties.opacity, 0.76)
      view.dispatch(view.find('FocusScope', (node) => node.properties.id === 'expressive-record-button'), 'click')
      await turn()
      assert(view.all('Text').some((node) => node.properties.text === 'Processing…'))
      view.dispatch(findButton(view, 'Pause motion'), 'click')
      await turn()
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'expressive-dotted-spinner')?.properties.loopPlaying, false)
    } finally { dispose() }
  })

  test(`${adapter} exposes five additional widget pages from the sidebar`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      for (const label of ['Checkbox', 'Switch', 'Tabs', 'Slider', 'Progress']) {
        view.dispatch(findButton(view, label), 'click')
        await turn()
        assert(view.all('Text').some((node) => node.properties.text === label), `${label} page opens`)
      }
    } finally { dispose() }
  })

  test(`${adapter} InputField applies a native edit to controlled text`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'InputField'), 'click')
      await turn()
      const input = view.find('TextInput', (node) => node.properties.label === 'Name')
      assert(input, 'Name TextInput is mounted')
      view.dispatch(input, 'edit', { kind: 'edit', start: 0, end: 3, text: 'Oct' })
      await turn()
      assert(view.all('Text').some((node) => node.properties.text === 'Current value: Oct Lovelace'))
    } finally { dispose() }
  })

  test(`${adapter} Select opens, accepts an option, and updates its accessible value`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'Select'), 'click')
      await turn()
      let select = view.find('FocusScope', (node) => node.properties.role === 'comboBox')
      assert(select, 'combobox is mounted')
      view.dispatch(select, 'click')
      await turn()
      const languagePopup = view.find('PopupWindow', (node) => node.properties.id === 'language-select-popup')
      assert(languagePopup, 'standard select popup is mounted')
      assert.equal(descendant(languagePopup, 'Rectangle')?.properties.clip, true,
        'popup contents stay inside the rounded border')
      assert.equal(languagePopup.properties.placementOffset, undefined,
        'standard select keeps the native anchored offset')
      assert.equal(descendant(languagePopup, 'ScrollView')?.properties.height, 158,
        'viewport includes option rows, gaps, and vertical padding')
      assert.equal(descendant(languagePopup, 'ScrollView')?.properties.scrollY, true,
        'long option lists can scroll within the border')
      const option = view.find('FocusScope', (node) => node.properties.role === 'option'
        && node.properties.accessibleName === 'TypeScript')
      assert(option, 'TypeScript option is mounted')
      view.dispatch(option, 'click')
      await turn()
      select = view.find('FocusScope', (node) => node.properties.role === 'comboBox')
      assert.equal(select?.properties.accessibleValue, 'TypeScript')

      let fruitSelect = view.find('FocusScope', (node) => node.properties.id === 'fruit-select')
      assert(fruitSelect, 'shadcn combobox is mounted')
      assert(view.text(fruitSelect).includes('Select a fruit'), 'closed trigger shows its placeholder')
      assert(!view.all('Text').some((node) => node.properties.text === 'Fruits'),
        'group title is hidden while the menu is closed')
      view.dispatch(fruitSelect, 'click')
      await turn()
      let fruitPopup = view.find('PopupWindow', (node) => node.properties.id === 'fruit-select-popup')
      assert(fruitPopup, 'shadcn popup is mounted')
      assert.equal(fruitPopup.properties.placementOffset, -61,
        'placeholder row is centered on the trigger when the menu fits')
      assert.equal(descendant(fruitPopup, 'ScrollView')?.properties.height, 212,
        'compact menu fits its title, placeholder, options, gaps, and padding')
      assert(view.text(fruitPopup).includes('Fruits'), 'group title appears in the open menu')
      let placeholder = view.find('FocusScope', (node) => node.properties.id === 'fruit-select-placeholder')
      assert.equal(placeholder?.properties.selected, true, 'empty value selects the placeholder row')
      const blueberry = view.find('FocusScope', (node) => node.properties.role === 'option'
        && node.properties.accessibleName === 'Blueberry')
      assert(blueberry, 'fruit option is mounted')
      view.dispatch(blueberry, 'click')
      await turn()
      fruitSelect = view.find('FocusScope', (node) => node.properties.id === 'fruit-select')
      assert.equal(fruitSelect?.properties.accessibleValue, 'Blueberry')
      assert(view.text(fruitSelect).includes('Blueberry'), 'chosen option replaces the trigger placeholder')
      view.dispatch(fruitSelect, 'click')
      await turn()
      fruitPopup = view.find('PopupWindow', (node) => node.properties.id === 'fruit-select-popup')
      assert.equal(fruitPopup?.properties.placementOffset, -151,
        'selected Blueberry row is centered on the trigger when reopened')
      const selectedBlueberry = view.find('FocusScope', (node) => node.properties.role === 'option'
        && node.properties.accessibleName === 'Blueberry')
      assert.equal(selectedBlueberry?.properties.selected, true, 'reopened menu marks the selected option')
      assert(view.text(selectedBlueberry).includes('✓'), 'selected row displays a native checkmark')
      placeholder = view.find('FocusScope', (node) => node.properties.id === 'fruit-select-placeholder')
      view.dispatch(placeholder, 'click')
      await turn()
      fruitSelect = view.find('FocusScope', (node) => node.properties.id === 'fruit-select')
      assert.equal(fruitSelect?.properties.accessibleValue, '', 'placeholder clears the controlled value')
    } finally { dispose() }
  })

  test(`${adapter} Popover mounts anchored content and closes on native dismissal`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'Popover'), 'click')
      await turn()
      const trigger = view.find('FocusScope', (node) => node.properties.controls === 'gallery-popover-popup')
      assert(trigger, 'anchored trigger is mounted')
      assert.equal(trigger.properties.hasPopup, undefined, 'generic popovers do not claim dialog semantics')
      assert.equal(descendant(trigger, 'Rectangle')?.properties.pressedTranslateY, undefined,
        'popup triggers do not use the active press translation')
      view.dispatch(trigger, 'click')
      await turn()
      let popup = view.find('PopupWindow', (node) => node.properties.id === 'gallery-popover-popup')
      assert(popup, 'native popup is mounted')
      assert.equal(popup.properties.anchor, 'gallery-popover')
      assert.equal(popup.properties.containment, 'none')
      assert.equal(popup.properties.initialFocus, '')
      assert.equal(popup.properties.restoreFocus, true)
      assert.equal(popup.properties.dismissPolicy, 'outsidePointerOrEscape')
      assert.equal(popup.properties.width, 240)
      assert.equal(trigger.properties.accessibleName, 'Opaque')
      assert.equal(trigger.properties.width, undefined)
      assert(view.all('Text').some((node) => node.properties.text === 'The background is fully opaque.'))
      view.dispatch(popup, 'dismiss')
      await turn()
      popup = view.find('PopupWindow', (node) => node.properties.id === 'gallery-popover-popup')
      assert.equal(popup, undefined)
      const searchTrigger = view.find('FocusScope', (node) => node.properties.controls === 'gallery-blurred-popover-popup')
      assert(searchTrigger, 'blurred filter trigger is mounted')
      view.dispatch(searchTrigger, 'click')
      await turn()
      const searchPopup = view.find('PopupWindow', (node) => node.properties.id === searchTrigger.properties.controls)
      assert(searchPopup, 'filter popup keeps its stable trigger relation')
      assert.equal(searchPopup.properties.width, 280)
      assert.equal(searchPopup.properties.initialFocus, 'first')
      assert.equal(searchPopup.properties.containment, 'none')
      assert.equal(searchPopup.properties.accessibleName, 'Blurred')
      assert.equal(searchTrigger.properties.width, undefined)
    } finally { dispose() }
  })

  test(`${adapter} VirtualList changes only the native requested item window`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'VirtualList'), 'click')
      await turn()
      const list = view.find('VirtualWindow')
      assert(list, 'native VirtualWindow is mounted')
      assert.equal(list.properties.itemCount, 5000)
      assert.equal(list.properties.variableHeight, false, 'fixed rows avoid measurement jitter')
      assert.equal(list.properties.overscan, 8, 'fast scroll keeps a larger bounded window')
      assert.equal(list.properties.scrollMomentum, 0.35)
      assert(list.children.length <= 12, `initial range is bounded (${list.children.length} rows)`)
      const createdBefore = view.all('Text').length
      view.dispatch(list, 'window', { kind: 'window', start: 100, end: 104, offset: 3600, viewportExtent: 430 })
      await turn()
      assert.equal(list.children.length, 4)
      assert(view.all('Text').some((node) => node.properties.text === 'Record 101'))
      assert(view.all('Text').length <= createdBefore + 4)
    } finally { dispose() }
  })

  test(`${adapter} scrollbar example exposes native hover geometry and momentum controls`, async () => {
    const view = galleryBridge()
    const dispose = mount(view.bridge, contract.abiHash)
    try {
      view.dispatch(findButton(view, 'Scrollbars'), 'click')
      await turn()
      const scrollbar = () => view.find('ScrollView', (node) => node.properties.id === 'gallery-custom-scrollbar')
      assert.equal(scrollbar()?.properties.scrollbarWidth, 6)
      assert.equal(scrollbar()?.properties.scrollbarHoverWidth, 10)
      assert.equal(scrollbar()?.properties.scrollbarSide, 'right')
      assert.equal(scrollbar()?.properties.scrollMomentum, 0.35)
      assert(scrollbar()?.properties.scrollbarTrackColor)
      assert(scrollbar()?.properties.scrollbarPressedColor)
      view.dispatch(findButton(view, 'Left'), 'click')
      await turn()
      assert.equal(scrollbar()?.properties.scrollbarSide, 'left')
      view.dispatch(findButton(view, '8 px'), 'click')
      view.dispatch(findButton(view, '+4 px on hover'), 'click')
      view.dispatch(findButton(view, 'Direct'), 'click')
      await turn()
      assert.equal(scrollbar()?.properties.scrollbarWidth, 8)
      assert.equal(scrollbar()?.properties.scrollbarHoverWidth, 8)
      assert.equal(scrollbar()?.properties.scrollbarSide, 'left')
      assert.equal(scrollbar()?.properties.scrollMomentum, 0)
    } finally { dispose() }
  })
}
