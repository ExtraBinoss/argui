import { mountGallery } from '../../apps/gallery/src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 800, height: 600, scale: 1 },
  async run(ui) {
    const window = ui.window('main')
    if ((await window.info()).width !== 800) throw new Error('initial window width is incorrect')
    await ui.getById('gallery-search').fill('Animation Lab')
    await ui.getById('page-animation-lab').click()
    await ui.expectText('Native loops run on the UI clock')
    await ui.wait(100)
    await ui.screenshot('resize-before.png')

    for (const [width, height] of [[1024, 720], [640, 480], [1280, 800], [800, 600]]) {
      await window.resize({ width, height })
      const actual = await window.info()
      if (actual.width !== width || actual.height !== height) {
        throw new Error(`resize ${width}x${height} settled at ${actual.width}x${actual.height}`)
      }
      await ui.screenshot(`resize-${width}x${height}.png`)
    }

    for (let frame = 1; frame <= 20; frame++) {
      await window.resize({ width: 800 + frame * 16, height: 600 + frame * 8 })
      await ui.wait(16)
    }
    await ui.screenshot('resize-drag-final.png')
  },
})
