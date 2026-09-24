import assert from 'node:assert/strict'
import { test } from 'node:test'
import { isDeepStrictEqual } from 'node:util'
import { mountGallery } from '../dist/gallery-core.mjs'
import { mountReactGallery } from '../dist/gallery-react-core.mjs'
import assets from '../assets.generated.json' with { type: 'json' }
import contract from '../../../packages/host/src/contract.generated.json' with { type: 'json' }

function capture(mount) {
  const batches = []
  let deliver = () => {}
  const dispose = mount({
    contract: () => contract,
    commit: (operations) => batches.push([...operations]),
    subscribe: (callback) => { deliver = callback; return () => { deliver = () => {} } },
  }, contract.abiHash)
  return { batches, dispose, deliver: (event) => deliver(event) }
}

function replay(batches) {
  const types = new Map(contract.natives.map((native) => [native.id, native]))
  const nodes = new Map()
  let root = null
  const id = (value) => `${value.slot}:${value.generation}`
  for (const operation of batches.flat()) {
    switch (operation.kind) {
      case 'create': nodes.set(id(operation.id), { type: types.get(operation.nativeType), properties: new Map(), listeners: new Map(), children: [] }); break
      case 'setProperty': {
        const node = nodes.get(id(operation.id))
        if (operation.value) node.properties.set(operation.property, operation.value)
        else node.properties.delete(operation.property)
        break
      }
      case 'setListener': {
        const node = nodes.get(id(operation.id))
        if (operation.callback === null) node.listeners.delete(operation.event)
        else node.listeners.set(operation.event, operation.callback)
        break
      }
      case 'insert': {
        const child = id(operation.child)
        for (const node of nodes.values()) node.children = node.children.filter((entry) => entry !== child)
        const parent = nodes.get(id(operation.parent))
        const index = operation.before ? parent.children.indexOf(id(operation.before)) : parent.children.length
        parent.children.splice(index, 0, child)
        break
      }
      case 'remove': {
        const target = id(operation.id)
        for (const node of nodes.values()) node.children = node.children.filter((entry) => entry !== target)
        const removeSubtree = (nodeId) => {
          const node = nodes.get(nodeId)
          if (!node) return
          for (const child of node.children) removeSubtree(child)
          nodes.delete(nodeId)
        }
        removeSubtree(target)
        break
      }
      case 'setRoot': root = operation.id ? id(operation.id) : null; break
    }
  }
  const canonical = (nodeId) => {
    const node = nodes.get(nodeId)
    const properties = Object.fromEntries([...node.properties].map(([propertyId, value]) => [
      node.type.properties.find((entry) => entry.id === propertyId).name, value,
    ]).sort(([left], [right]) => left.localeCompare(right)))
    const listeners = [...node.listeners.keys()].map((eventId) => node.type.events.find((entry) => entry.id === eventId).name).sort()
    return {
      type: node.type.name,
      properties,
      listeners,
      children: node.children.map(canonical).filter((child) => child.type !== 'Text' || child.properties.text?.value !== ''),
    }
  }
  return { tree: canonical(root), nodes }
}

function activate(capture, key) {
  const { nodes } = replay(capture.batches)
  const target = [...nodes].find(([, node]) => node.properties.get(1)?.value === key)
  assert.ok(target, `native key ${key} must exist`)
  const [identity, node] = target
  const event = node.type.events.find((entry) => entry.name === 'click')
  assert.ok(event, `${key} must support click`)
  const callback = node.listeners.get(event.id)
  assert.ok(callback, `${key} must have a click listener`)
  const [slot, generation] = identity.split(':').map(Number)
  capture.deliver({ node: { slot, generation }, callback })
}

function edit(capture, key, eventName, text) {
  const { nodes } = replay(capture.batches)
  const target = [...nodes].find(([, node]) => node.properties.get(1)?.value === key)
  assert.ok(target, `native key ${key} must exist`)
  const [identity, node] = target
  const event = node.type.events.find((entry) => entry.name === eventName)
  assert.ok(event, `${key} must support ${eventName}`)
  const callback = node.listeners.get(event.id)
  assert.ok(callback, `${key} must have a ${eventName} listener`)
  const [slot, generation] = identity.split(':').map(Number)
  capture.deliver({ node: { slot, generation }, callback, payload: { kind: eventName, text } })
}

function nativeIdentity(capture, key) {
  const { nodes } = replay(capture.batches)
  const entry = [...nodes].find(([, node]) => node.properties.get(1)?.value === key)
  assert.ok(entry, `native key ${key} must remain mounted`)
  return entry[0]
}

const animatedProperties = new Set(['x', 'y', 'width', 'height', 'gap', 'rotation', 'opacity', 'radius', 'background', 'color'])

function stableTree(node) {
  const animated = node.properties.transition_ms || node.properties.transition_spring
  const properties = Object.fromEntries(Object.entries(node.properties).map(([name, value]) => [
    name,
    animated && animatedProperties.has(name) ? { ...value, value: '<animated>' } : value,
  ]))
  return { ...node, properties, children: node.children.map(stableTree) }
}

/** Counts scrolling ancestors of one keyed control in the native tree. */
function scrollingAncestors(root, key) {
  let result = null
  const visit = (node, ancestors) => {
    if (node.properties.key?.value === key) result = ancestors
    const next = ancestors + Number(node.properties.scroll_y?.value === true)
    for (const child of node.children) visit(child, next)
  }
  visit(root, 0)
  assert.notEqual(result, null, `native key ${key} must exist`)
  return result
}

function activationStatus(node) {
  if (node.type === 'Text' && node.properties.text?.value.startsWith('Clicks:')) {
    return node.properties.text.value
  }
  for (const child of node.children) {
    const status = activationStatus(child)
    if (status) return status
  }
  return null
}

function assertPaintOnlyContinuousLayoutDemos(root) {
  const nodes = []
  const visit = (node) => {
    nodes.push(node)
    for (const child of node.children) visit(child)
  }
  visit(root)
  assert.ok(nodes.every((node) => !node.properties.loop_width && !node.properties.loop_gap))
  assert.ok(nodes.some((node) => node.type === 'Rectangle'
    && node.properties.width && node.properties.transition_ms && node.properties.loop_scale))
  assert.ok(nodes.some((node) => node.type === 'Row'
    && node.properties.gap && node.properties.transition_ms
    && node.children.filter((child) => child.properties.loop_translate_x).length === 2))
}

function keyedText(root, key) {
  if (root.properties.key?.value === key) {
    const firstText = (node) => node.type === 'Text' ? node.properties.text?.value
      : node.children.map(firstText).find((value) => value !== undefined)
    return firstText(root)
  }
  return root.children.map((child) => keyedText(child, key)).find((value) => value !== undefined)
}

function allNodes(root) {
  return [root, ...root.children.flatMap(allNodes)]
}

test('Solid and React keep native structure, stable properties, and blocked button actions in parity', async () => {
  const solid = capture(mountGallery)
  const react = capture(mountReactGallery)
  try {
    const tree = (capture) => stableTree(replay(capture.batches).tree)
    const compare = (phase) => assert.deepEqual(tree(react), tree(solid), phase)
    const click = async (key) => {
      activate(solid, key)
      activate(react, key)
      for (let attempt = 0; attempt < 100; attempt++) {
        await new Promise((resolve) => setTimeout(resolve, 10))
        if (isDeepStrictEqual(tree(react), tree(solid))) break
      }
      compare(key)
    }
    compare('initial Button page')
    const type = async (key, eventName, value) => {
      edit(solid, key, eventName, value)
      edit(react, key, eventName, value)
      for (let attempt = 0; attempt < 100; attempt++) {
        await new Promise((resolve) => setTimeout(resolve, 10))
        if (isDeepStrictEqual(tree(react), tree(solid))) break
      }
      compare(`${key} ${eventName}`)
    }
    const searchIdentities = [solid, react].map((presentation) => nativeIdentity(presentation, 'gallery-search'))
    for (const value of ['i', 'in', 'inp']) {
      await type('gallery-search', 'input', value)
      for (const [index, presentation] of [solid, react].entries()) {
        assert.equal(nativeIdentity(presentation, 'gallery-search'), searchIdentities[index],
          `search input must retain focusable native identity after ${value}`)
        const searchNode = [...replay(presentation.batches).nodes.values()]
          .find((node) => node.properties.get(1)?.value === 'gallery-search')
        const clipProperty = searchNode.type.properties.find((property) => property.name === 'clip')
        assert.equal(searchNode.properties.get(clipProperty.id)?.value, true)
        assert.equal(searchNode.properties.get(searchNode.type.properties.find((property) => property.name === 'selection_color').id)?.value,
          '#2563eb61')
      }
    }
    for (const presentation of [solid, react]) {
      const current = tree(presentation)
      const navigation = allNodes(current).find((node) => node.properties.key?.value === 'gallery-navigation')
      assert.equal(navigation?.properties.__item_count?.value, 2)
      assert.ok(allNodes(current).some((node) => node.properties.key?.value === 'page-input'))
      assert.ok(!allNodes(current).some((node) => node.properties.key?.value === 'page-button'))
    }
    await click('page-input')
    await type('input-name', 'input', 'Ada')
    await type('input-name', 'submit', 'Ada')
    await type('input-password', 'input', 'secret')
    const passwordIdentities = [solid, react].map((presentation) => nativeIdentity(presentation, 'input-password'))
    for (const presentation of [solid, react]) {
      const node = allNodes(tree(presentation)).find((candidate) => candidate.properties.key?.value === 'input-password')
      assert.equal(node?.properties.privacy?.value, 'password')
    }
    await click('input-password-visibility')
    for (const [index, presentation] of [solid, react].entries()) {
      assert.equal(nativeIdentity(presentation, 'input-password'), passwordIdentities[index])
      const node = allNodes(tree(presentation)).find((candidate) => candidate.properties.key?.value === 'input-password')
      assert.equal(node?.properties.privacy?.value, 'revealed_password')
    }
    await click('input-password-visibility')
    for (const presentation of [solid, react]) {
      const node = allNodes(tree(presentation)).find((candidate) => candidate.properties.key?.value === 'input-password')
      assert.equal(node?.properties.privacy?.value, 'password')
    }
    for (const presentation of [solid, react]) {
      assert.ok(allNodes(tree(presentation)).some((node) => node.type === 'Text'
        && node.properties.text?.value === 'Submitted: Ada'))
    }
    await type('gallery-search', 'input', '')
    await click('page-button')
    activate(solid, 'button-disabled')
    activate(react, 'button-disabled')
    activate(solid, 'button-busy')
    activate(react, 'button-busy')
    await new Promise((resolve) => setTimeout(resolve, 20))
    assert.equal(activationStatus(tree(solid)), 'Clicks: 0 · Last used: None')
    assert.equal(activationStatus(tree(react)), 'Clicks: 0 · Last used: None')
    await click('button-primary')
    await click('theme-toggle')
    await click('accent-violet')
    await click('page-select')
    await click('topic-select')
    for (const presentation of [solid, react]) {
      const popup = allNodes(tree(presentation)).find((node) => node.properties.key?.value === 'topic-select-popup')
      assert.equal(popup?.type, 'PopupWindow')
      assert.ok(allNodes(popup).some((node) => node.properties.background?.value === '#1a1b23'
        && node.properties.shadow_color?.value === '#00000066'))
      assert.ok(allNodes(popup).every((node) => !node.properties.backdrop_filter))
    }
    await click('topic-select-option-2')
    for (const presentation of [solid, react]) {
      assert.equal(keyedText(tree(presentation), 'topic-select'), 'Metal')
    }
    await click('page-popover')
    await click('demo-popover')
    for (const presentation of [solid, react]) {
      const current = tree(presentation)
      const popup = allNodes(current).find((node) => node.properties.key?.value === 'demo-popover-popup')
      assert.equal(popup?.type, 'PopupWindow')
      assert.equal(popup.properties.anchor?.value, 'demo-popover')
      assert.equal(popup.properties.window_layer?.value, 'popover')
      assert.ok(allNodes(popup).some((node) => node.properties.backdrop_filter?.value === 'blur(10px)'
        && node.properties.shadow_color?.value === '#00000066'))
      assert.equal(allNodes(current).find((node) => node.properties.key?.value === 'demo-popover')
        ?.properties.expanded?.value, true)
    }
    await click('demo-popover-solid')
    for (const presentation of [solid, react]) {
      const nodes = allNodes(tree(presentation))
      const popup = nodes.find((node) => node.properties.key?.value === 'demo-popover-solid-popup')
      assert.equal(popup?.type, 'PopupWindow')
      assert.ok(!nodes.some((node) => node.properties.key?.value === 'demo-popover-popup'))
      assert.ok(allNodes(popup).some((node) => node.properties.shadow_color?.value === '#00000066'
        && node.properties.border_color?.value === '#383b4b'
        && node.properties.background?.value === '#1a1b23d0'))
      assert.ok(allNodes(popup).every((node) => !node.properties.backdrop_filter))
    }
    await click('demo-popover-opaque')
    for (const presentation of [solid, react]) {
      const nodes = allNodes(tree(presentation))
      const popup = nodes.find((node) => node.properties.key?.value === 'demo-popover-opaque-popup')
      assert.equal(popup?.type, 'PopupWindow')
      assert.ok(!nodes.some((node) => node.properties.key?.value === 'demo-popover-solid-popup'))
      assert.ok(allNodes(popup).some((node) => node.properties.shadow_color?.value === '#00000066'
        && node.properties.border_color?.value === '#383b4b'
        && node.properties.background?.value === '#1a1b23'))
      assert.ok(allNodes(popup).every((node) => !node.properties.backdrop_filter))
    }
    await click('demo-popover-opaque-close')
    await click('page-dialog')
    await click('dialog-open')
    for (const presentation of [solid, react]) {
      const nodes = allNodes(tree(presentation))
      const dialog = nodes.find((node) => node.properties.key?.value === 'gallery-dialog-dialog')
      assert.equal(dialog?.properties.window_layer?.value, 'modal')
      assert.equal(dialog?.properties.placement?.value, 'fill')
      assert.equal(dialog?.properties.containment?.value, 'modal')
      assert.ok(allNodes(dialog).some((node) => node.properties.backdrop_filter?.value === 'blur(18px)'))
    }
    await click('gallery-dialog-close')
    await type('gallery-search', 'input', 'Animation')
    await click('page-animation-lab')
    for (const presentation of [solid, react]) {
      const current = tree(presentation)
      assert.equal(scrollingAncestors(current, 'page-animation-lab'), 0)
      assert.equal(scrollingAncestors(current, 'motion-target'), 1)
      assertPaintOnlyContinuousLayoutDemos(current)
    }
    await click('motion-target')
    for (const presentation of [solid, react]) assertPaintOnlyContinuousLayoutDemos(tree(presentation))
    await click('motion-step')
    await click('motion-play')
    await click('motion-play')
    await type('gallery-search', 'input', '')
    await click('page-media')
    const orbit = assets.assets.find((asset) => asset.key === 'illustration/orbit.png')
    const saturn = assets.assets.find((asset) => asset.key === 'photo/saturn.jpg')
    assert.ok(orbit && saturn)
    for (const presentation of [solid, react]) {
      const nodes = allNodes(tree(presentation))
      const mediaPane = nodes.find((node) => node.properties.key?.value === 'scroll-Media')
      assert.ok(mediaPane)
      assert.deepEqual(allNodes(mediaPane).filter((node) => node.type === 'Image')
        .map((node) => node.properties.source?.value.id).sort(), [orbit.id, saturn.id].sort())
      assert.equal(allNodes(mediaPane).filter((node) => node.type === 'Svg').length, 3)
      const navigation = nodes.find((node) => node.type === 'VirtualWindow'
        && node.properties.key?.value === 'gallery-navigation')
      assert.ok(navigation, 'gallery navigation must use the native virtual list')
      assert.ok(navigation.children.length <= 12, 'only a bounded navigation range mounts')
      assert.ok(nodes.some((node) => node.type === 'Text' && node.properties.text?.value === 'EXAMPLES'))
      const overlayPane = nodes.find((node) => node.properties.key?.value === 'scroll-Overlay')
      assert.ok(overlayPane)
      assert.equal(allNodes(overlayPane).filter((node) => node.properties.backdrop_filter?.value).length, 12)
    }
    await type('gallery-search', 'input', 'Theming')
    await click('page-theming')
    await click('theme-preset-Paper')
    await click('theme-blur')
    for (const presentation of [solid, react]) {
      const pane = allNodes(tree(presentation)).find((node) => node.properties.key?.value === 'scroll-Theming')
      assert.ok(allNodes(pane).some((node) => node.type === 'Rectangle'
        && node.properties.backdrop_filter?.value === 'blur(10px)'
        && node.properties.background?.value === '#fffdf899'))
    }
    const customTheme = { canvas: '#eee9df', panel: '#fffdf899', ink: '#282e35', muted: '#69737d',
      accent: '#123456', radius: 12, padding: 10, blur: 7, fontSize: 21,
      shadowBlur: 5, shadowColor: '#282e3544', appSpecificVariable: 31 }
    await type('theme-json', 'input', JSON.stringify(customTheme))
    await click('theme-load-json')
    for (const presentation of [solid, react]) {
      const pane = allNodes(tree(presentation)).find((node) => node.properties.key?.value === 'scroll-Theming')
      assert.ok(pane)
      const nodes = allNodes(pane)
      assert.ok(nodes.some((node) => node.type === 'Text' && node.properties.text?.value === 'Themed text surface'
        && node.properties.padding?.value === 10 && node.properties.radius?.value === 12
        && node.properties.background?.value === '#123456'))
      assert.ok(nodes.some((node) => node.type === 'Rectangle'
        && node.properties.backdrop_filter?.value === 'blur(7px)'
        && node.properties.background?.value === '#fffdf899'))
    }
    await type('gallery-search', 'input', 'Overlay')
    await click('page-overlay')
    await click('menu')
    await click('menu')
  } finally {
    solid.dispose()
    react.dispose()
  }
})

test('mobile Solid and React keep navigation fixed above the content scroll', async () => {
  const previous = globalThis.__arguiMobile
  globalThis.__arguiMobile = true
  let solid
  let react
  try {
    const { mountGallery: mountMobileSolid } = await import('../dist/gallery-core.mjs?mobile-navigation')
    const { mountReactGallery: mountMobileReact } = await import('../dist/gallery-react-core.mjs?mobile-navigation')
    solid = capture(mountMobileSolid)
    react = capture(mountMobileReact)
    edit(solid, 'gallery-search', 'input', 'Animation')
    edit(react, 'gallery-search', 'input', 'Animation')
    for (let attempt = 0; attempt < 100; attempt++) {
      await new Promise((resolve) => setTimeout(resolve, 10))
      if ([solid, react].every((presentation) => [...replay(presentation.batches).nodes.values()]
        .some((node) => node.properties.get(1)?.value === 'page-animation-lab'))) break
    }
    activate(solid, 'page-animation-lab')
    activate(react, 'page-animation-lab')
    for (let attempt = 0; attempt < 100; attempt++) {
      await new Promise((resolve) => setTimeout(resolve, 10))
      const solidTree = stableTree(replay(solid.batches).tree)
      const reactTree = stableTree(replay(react.batches).tree)
      if (isDeepStrictEqual(reactTree, solidTree)) break
    }
    for (const presentation of [solid, react]) {
      const tree = replay(presentation.batches).tree
      assert.equal(scrollingAncestors(tree, 'page-animation-lab'), 0)
      assert.equal(scrollingAncestors(tree, 'motion-target'), 1)
      const navigation = allNodes(tree).find((node) => node.type === 'VirtualWindow'
        && node.properties.key?.value === 'gallery-navigation')
      assert.equal(navigation?.properties.horizontal?.value, true)
    }
  } finally {
    solid?.dispose()
    react?.dispose()
    if (previous === undefined) delete globalThis.__arguiMobile
    else globalThis.__arguiMobile = previous
  }
})
