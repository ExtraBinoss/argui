import { mountGallery } from '../../apps/counter-native/src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 800, height: 600, scale: 1 },
  async run(ui) {
    await ui.expectText('Count: 0')
    await ui.getById('increment').click()
    await ui.getById('increment').click()
    await ui.getById('increment').click()
    await ui.expectText('Count: 3')
    let timerFired = false
    setTimeout(() => { timerFired = true }, 20)
    await ui.wait(50)
    if (!timerFired) throw new Error('JavaScript timers did not advance during ui.wait')
    await ui.screenshot('counter.png')
  },
})
