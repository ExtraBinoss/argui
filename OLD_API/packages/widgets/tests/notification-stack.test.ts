import { expect, test } from 'bun:test'
import { notificationStore, ToastStore, toast } from '../src/shared/notification-stack'

const delay = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds))

test('notification stores push and dismiss one or all records', () => {
  const store = new ToastStore()
  const first = store.push('Saved', { duration: 0 }, 'success')
  const second = store.push('Reminder', { duration: 0 }, 'info')
  expect(store.getSnapshot().map((record) => record.id)).toEqual([first, second])

  store.dismiss(first)
  expect(store.getSnapshot().map((record) => record.title)).toEqual(['Reminder'])
  store.dismiss()
  expect(store.getSnapshot()).toEqual([])
})

test('hover pause preserves the remaining automatic-dismiss time', async () => {
  const store = new ToastStore()
  const id = store.push('Read this first', { duration: 70 })
  store.pause(id)
  await delay(100)
  expect(store.getSnapshot()).toHaveLength(1)

  store.resume(id)
  await delay(110)
  expect(store.getSnapshot()).toEqual([])
})

test('promise notifications transition in place on success and error', async () => {
  toast.dismiss()
  let resolveUpload!: (value: string) => void
  const upload = new Promise<string>((resolve) => { resolveUpload = resolve })
  const saved = toast.promise(upload, {
    loading: 'Uploading',
    success: (filename) => `${filename} uploaded`,
    error: 'Upload failed',
    duration: 0,
  })
  const pending = notificationStore.getSnapshot()[0]
  expect(pending?.variant).toBe('loading')
  expect(pending?.dismissible).toBe(false)
  resolveUpload('report.pdf')
  await saved
  expect(notificationStore.getSnapshot()[0]).toMatchObject({
    id: pending?.id,
    title: 'report.pdf uploaded',
    variant: 'success',
    dismissible: true,
  })

  const failure = new Error('network unavailable')
  let caught: unknown
  try {
    await toast.promise(Promise.reject(failure), {
      loading: 'Retrying', success: 'Uploaded', error: (error) => String(error), duration: 0,
    })
  } catch (error) {
    caught = error
  }
  expect(caught).toBe(failure)
  expect(notificationStore.getSnapshot()[1]).toMatchObject({ title: String(failure), variant: 'error' })
  toast.dismiss()
})
