import { expect, test } from 'bun:test'
import { ApplicationServices, ServiceError, type NativeBridge, type ServiceRequest, type ServiceResponse } from '../src'

function bridge() {
  const requests: ServiceRequest[] = []
  const cancellations: [string, number][] = []
  const listeners = new Set<(response: ServiceResponse) => void>()
  const native: NativeBridge = {
    contract: () => ({ abiHash: '1', natives: [] }),
    commit: () => {}, subscribe: () => () => {},
    request: (request) => { requests.push(request) },
    cancelRequest: (window, requestId) => { cancellations.push([window, requestId]) },
    subscribeResponses: (listener) => { listeners.add(listener); return () => { listeners.delete(listener) } },
  }
  return {
    native, requests, cancellations,
    deliver: (response: ServiceResponse) => { for (const listener of listeners) listener(response) },
    listenerCount: () => listeners.size,
  }
}

test('responses settle only the matching window and request', async () => {
  const mock = bridge()
  const main = new ApplicationServices(mock.native)
  const second = new ApplicationServices(mock.native, 'preview')
  const first = main.readClipboardText()
  const other = second.readClipboardText()
  expect(mock.requests).toEqual([
    { requestId: 1, window: 'main', service: 'clipboard', method: 'readText', payload: null },
    { requestId: 1, window: 'preview', service: 'clipboard', method: 'readText', payload: null },
  ])
  mock.deliver({ requestId: 1, window: 'preview', status: 'ok', value: 'secondary' })
  mock.deliver({ requestId: 1, window: 'main', status: 'ok', value: 'primary' })
  expect(await first).toBe('primary')
  expect(await other).toBe('secondary')
  main.dispose()
  second.dispose()
})

test('abort and disposal cancel pending native work and ignore late replies', async () => {
  const mock = bridge()
  const services = new ApplicationServices(mock.native)
  const controller = new AbortController()
  const aborted = services.openFiles({}, controller.signal)
  controller.abort()
  expect(aborted).rejects.toMatchObject({ code: 'cancelled' })
  const pending = services.saveFile()
  services.dispose()
  expect(pending).rejects.toMatchObject({ code: 'closed' })
  expect(mock.cancellations).toEqual([['main', 1], ['main', 2]])
  expect(mock.listenerCount()).toBe(0)
  mock.deliver({ requestId: 2, window: 'main', status: 'ok', value: { path: '/tmp/late', name: 'late' } })
  expect(services.readClipboardText()).rejects.toBeInstanceOf(ServiceError)
})

test('native errors and unsupported methods keep their distinct codes', async () => {
  const mock = bridge()
  const services = new ApplicationServices(mock.native)
  const unsupported = services.call('menus', 'set', {})
  mock.deliver({ requestId: 1, window: 'main', status: 'unsupported', message: 'No menu backend' })
  expect(unsupported).rejects.toMatchObject({ code: 'unsupported', message: 'No menu backend' })
  const failed = services.readClipboardText()
  mock.deliver({ requestId: 2, window: 'main', status: 'error', message: 'Permission denied' })
  expect(failed).rejects.toMatchObject({ code: 'error', message: 'Permission denied' })
  services.dispose()
})

test('main window sends a message and receives a companion reply', async () => {
  const mock = bridge()
  const services = new ApplicationServices(mock.native)
  const events: string[] = []
  const unsubscribe = services.onEvent((event) => {
    if (event.type === 'window') events.push(`${event.window}: ${event.message}`)
  })
  const opened = services.call<void>('windows', 'openCompanion', {
    title: 'Preview', width: 640, height: 360, decorations: false,
    transparent: true, backdrop: true, backdropScope: 'panel',
  })
  expect(mock.requests[0]).toMatchObject({ service: 'windows', method: 'openCompanion', window: 'main', payload: {
    title: 'Preview', width: 640, height: 360, decorations: false,
    transparent: true, backdrop: true, backdropScope: 'panel',
  } })
  mock.deliver({ requestId: 1, window: 'main', status: 'ok', value: null })
  await opened
  const sent = services.call<void>('windows', 'sendMessage', { message: 'burst from main' })
  expect(mock.requests[1]).toMatchObject({ service: 'windows', method: 'sendMessage', payload: { message: 'burst from main' } })
  mock.deliver({ requestId: 2, window: 'main', status: 'ok', value: null })
  await sent
  mock.deliver({ requestId: 0, window: 'main', status: 'event', value: {
    type: 'window', window: 'companion', message: 'Companion received: burst from main',
  } })
  expect(events).toEqual(['companion: Companion received: burst from main'])
  unsubscribe()
  services.dispose()
})

test('window details and live title, size, and decoration requests retain the target key', async () => {
  const mock = bridge()
  const services = new ApplicationServices(mock.native)
  const info = services.getWindowInfo('companion')
  expect(mock.requests[0]).toMatchObject({ service: 'windows', method: 'getInfo', payload: { window: 'companion' } })
  mock.deliver({ requestId: 1, window: 'main', status: 'ok', value: {
    window: 'companion', title: 'Preview', width: 640, height: 360,
    visible: true, decorations: false, transparent: true,
    backdrop: true, backdropAvailable: false,
  } })
  expect(await info).toMatchObject({ width: 640, backdropAvailable: false })
  const changed = [
    services.setWindowTitle('companion', 'Renamed'),
    services.setWindowSize('companion', 800, 500),
    services.setWindowDecorations('companion', true),
  ]
  expect(mock.requests.slice(1)).toMatchObject([
    { method: 'setTitle', payload: { window: 'companion', title: 'Renamed' } },
    { method: 'setSize', payload: { window: 'companion', width: 800, height: 500 } },
    { method: 'setDecorations', payload: { window: 'companion', decorations: true } },
  ])
  for (let requestId = 2; requestId <= 4; requestId++) {
    mock.deliver({ requestId, window: 'main', status: 'ok', value: null })
  }
  await Promise.all(changed)
  services.dispose()
})
