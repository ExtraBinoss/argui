import { mountGallery } from '../../apps/gallery/src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 1024, height: 768, scale: 1 },
  async run(ui) {
    await ui.getById('gallery-search').fill('Animation Lab')
    await ui.getById('page-animation-lab').click()
    await ui.expectText('Native loops run on the UI clock')
    await ui.wait(400)

    for (let click = 0; click < 3; click++) await ui.getById('motion-target').click()
    let timerFired = false
    setTimeout(() => { timerFired = true }, 20)
    await ui.pause()
    await ui.screenshot('animation-lab-paused.png')
    await ui.sleep(50)
    if (timerFired) throw new Error('JavaScript timer advanced while paused')
    await ui.screenshot('animation-lab-frozen.png')
    await ui.resume()
    await ui.wait(900)
    if (!timerFired) throw new Error('JavaScript timer did not resume')

    const content = ui.getById('scroll-Animation Lab')
    for (let step = 0; step < 60; step++) {
      await content.scroll({ x: 0, y: step % 12 < 6 ? 300 : -300 })
      await ui.wait(16)
      if (step === 29) await ui.screenshot('animation-lab-middle.png')
    }
    await ui.screenshot('animation-lab-after.png')
  },
})
