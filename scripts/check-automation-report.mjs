import { readFileSync } from 'node:fs'
import { join } from 'node:path'

const dir = process.argv[2]
if (!dir) throw new Error('Usage: bun scripts/check-automation-report.mjs <artifact-dir>')
const report = JSON.parse(readFileSync(join(dir, 'report.json'), 'utf8'))
if (!report.ok) throw new Error(`Automation failed: ${report.error}`)
if (!report.steps?.every(step => step.ok)) throw new Error('A test step failed')
if (!report.artifacts?.includes('counter.png')) throw new Error('Counter capture is missing from the report')
if (!report.frameSummary?.count || !report.frames?.length) throw new Error('Argui frame samples are missing')
if (!report.metrics?.events?.length || !report.metrics?.phases?.['layout.compute']) {
  throw new Error('Nested engine timing metrics are missing')
}
if (!report.frameDiagnostics?.length || !report.slowFrames?.length) {
  throw new Error('Frame diagnostics are missing')
}
if (!report.steps.every(step => Number.isFinite(step.gap_ms) && Number.isInteger(step.trace_id))) {
  throw new Error('Action gaps or trace links are missing')
}
const wait = report.steps.find(step => step.name === 'wait')
if (!wait || wait.duration_ms < 40) throw new Error('Timed automation wait was not exercised')
const waitSpan = report.metrics.events.find(event => event.name === 'automation.wait' && event.parentId === wait.trace_id)
if (!waitSpan || !report.metrics.events.some(event => event.name === 'js.tick' && event.parentId === waitSpan.id)) {
  throw new Error('JavaScript timers did not run inside the wait action')
}
if (!report.frameDiagnostics.some(frame => frame.renderWorkload?.drawBatches > 0)) {
  throw new Error('Renderer workload counters are missing')
}
if (!report.processMetrics?.hostPid) throw new Error('Host PID is missing')
const samples = report.processMetrics.samples ?? []
if (!samples.some(sample => sample.cpuPercent !== null && Number.isFinite(sample.cpuPercent))) {
  throw new Error('Process CPU samples are missing')
}
if (!samples.some(sample => sample.rssBytes > 0)) throw new Error('Process memory samples are missing')
const png = readFileSync(join(dir, 'counter.png'))
if (png.length < 1000 || png.subarray(1, 4).toString() !== 'PNG') {
  throw new Error('Counter PNG is missing or invalid')
}
if (png.readUInt32BE(16) !== 800 || png.readUInt32BE(20) !== 600) {
  throw new Error('Counter PNG does not match the requested viewport')
}
console.log(`Automation report: ${report.steps.length} steps, ${samples.length} process samples, ${report.frames.length} frames`)
