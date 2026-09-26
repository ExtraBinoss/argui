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
      let surface = selected?.children[0]
      assert.equal(surface?.properties.background, 'oklch(0.97 0 0)', 'light selected navigation uses sidebar accent')
      assert.equal(descendant(selected, 'Text')?.properties.textColor, 'oklch(0.205 0 0)', 'selected navigation label uses sidebar primary')
      assert.equal(surface?.properties.hoverBackground, 'oklch(0.97 0 0)', 'ghost hover follows the official muted role')
      assert.equal(surface?.properties.pressedScale, 0.98, 'native press gently scales the control')

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
        'theme update changes existing native properties without rebuilding nodes')
      selected = view.find('FocusScope', (node) => node.properties.id === 'page-layouting')
      surface = selected?.children[0]
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
      assert.equal(schedule.children[0]?.properties.radii, 10,
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
      for (const id of ['animation-travel', 'animation-scale', 'animation-color', 'animation-width']) {
        const scene = view.find('Rectangle', (node) => node.properties.id === id)
        assert(scene, `${id} is mounted`)
        assert(descendant(scene, 'Text'), `${id} includes explanatory text`)
        assert(scene.properties.loopMs > 0, `${id} uses native loop timing`)
      }
      view.dispatch(findButton(view, 'Pause animations'), 'click')
      await turn()
      assert.equal(view.find('Rectangle', (node) => node.properties.id === 'animation-travel')?.properties.loopPlaying, false)
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
      assert.equal(languagePopup.properties.placementOffset, undefined,
        'standard select keeps the native anchored offset')
      assert.equal(descendant(languagePopup, 'ScrollView')?.properties.height, 158,
        'viewport includes option rows, gaps, and vertical padding')
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
      view.dispatch(trigger, 'click')
      await turn()
      let popup = view.find('PopupWindow', (node) => node.properties.id === 'gallery-popover-popup')
      assert(popup, 'native popup is mounted')
      assert.equal(popup.properties.anchor, 'gallery-popover')
      assert.equal(popup.properties.containment, 'none')
      assert.equal(popup.properties.initialFocus, '')
      assert.equal(popup.properties.restoreFocus, true)
      assert.equal(popup.properties.dismissPolicy, 'outsidePointerOrEscape')
      assert.equal(popup.properties.width, 280)
      assert.equal(trigger.properties.accessibleName, 'Show details')
      assert.equal(trigger.properties.width, undefined)
      assert(view.all('Text').some((node) => node.properties.text === 'Popover content'))
      view.dispatch(popup, 'dismiss')
      await turn()
      popup = view.find('PopupWindow', (node) => node.properties.id === 'gallery-popover-popup')
      assert.equal(popup, undefined)
      const searchTrigger = view.find('FocusScope', (node) => node.properties.accessibleName === 'Search filters')
      assert(searchTrigger, 'accessibleLabel names the filter trigger')
      view.dispatch(searchTrigger, 'click')
      await turn()
      const searchPopup = view.find('PopupWindow', (node) => node.properties.id === searchTrigger.properties.controls)
      assert(searchPopup, 'filter popup keeps its stable trigger relation')
      assert.equal(searchPopup.properties.width, 320)
      assert.equal(searchPopup.properties.initialFocus, 'first')
      assert.equal(searchPopup.properties.containment, 'none')
      assert.equal(searchPopup.properties.accessibleName, 'Search filters')
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
      assert(list.children.length <= 12, `initial range is bounded (${list.children.length} rows)`)
      const createdBefore = view.all('Text').length
      view.dispatch(list, 'window', { kind: 'window', start: 100, end: 104, offset: 3600, viewportExtent: 430 })
      await turn()
      assert.equal(list.children.length, 4)
      assert(view.all('Text').some((node) => node.properties.text === 'Record 101'))
      assert(view.all('Text').length <= createdBefore + 4)
    } finally { dispose() }
  })
}
