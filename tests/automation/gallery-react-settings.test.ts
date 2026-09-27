import { mountReactGallery } from '../../apps/gallery/src/react/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountReactGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    await ui.getById('gallery-settings').click()
    await ui.wait(20)
    await ui.expectText('Color family')
    await ui.getById('gallery-color-list').scroll({ x: 0, y: 2000 })
    await ui.wait(30)
    await ui.getById('color-rose').click()
    await ui.screenshot('react-settings-last-color.png')
  },
})
