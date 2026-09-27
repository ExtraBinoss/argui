import { mountGallery } from '../../apps/gallery/src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    await ui.getById('gallery-settings').click()
    await ui.expectText('Color family')
    await ui.screenshot('settings-open.png')
    await ui.getById('gallery-settings').click()
    await ui.getById('gallery-settings').click()
    await ui.getById('gallery-color-list').scroll({ x: 0, y: 2000 })
    await ui.wait(30)
    await ui.getById('color-rose').click()
    await ui.screenshot('settings-last-color.png')
  },
})
