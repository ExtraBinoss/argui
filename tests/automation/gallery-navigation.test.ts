import { mountGallery } from '../../apps/gallery/src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    const navigation = ui.getById('gallery-navigation')
    await ui.screenshot('gallery-navigation-start.png')
    for (let step = 0; step < 16; step++) {
      await navigation.scroll({ x: 0, y: 300 })
      await ui.wait(20)
    }
    await ui.screenshot('gallery-navigation-bottom.png')
    await ui.getById('page-wgsl-lab').click()
    await ui.expectText('WGSL Lab')
  },
})
