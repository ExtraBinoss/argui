import { mountGallery } from '../../apps/gallery/src/main'
import { defineArguiTest } from '@argui/test'

const rows = [
  'accordion', 'alert', 'alert-dialog', 'aspect-ratio', 'attachment',
  'avatar', 'badge', 'breadcrumb', 'bubble', 'button', 'button-group',
]

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    for (let pass = 0; pass < 8; pass++) {
      for (const row of rows) await ui.getById(`page-${row}`).move()
      for (const row of [...rows].reverse()) await ui.getById(`page-${row}`).move()
    }
    await ui.wait(40)
    await ui.screenshot('gallery-hover.png')
  },
})
