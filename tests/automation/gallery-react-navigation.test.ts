import { mountReactGallery } from '../../apps/gallery/src/react/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountReactGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    const navigation = ui.getById('gallery-navigation')
    for (let step = 0; step < 16; step++) {
      await navigation.scroll({ x: 0, y: 300 })
      await ui.wait(20)
    }
    await ui.screenshot('gallery-react-navigation-bottom.png')
    await ui.getById('page-wgsl-lab').click()
    await ui.expectText('WGSL Lab')
  },
})
