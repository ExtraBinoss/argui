/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Button, Progress, type WidgetTheme } from '@argui/widgets/react'

/** Demonstrates bounded progress and a user-driven update. */
export function ProgressPage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [progress, setProgress] = useState(35)
  const [playing, setPlaying] = useState(true)
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Progress</text>
    <text color={theme.textMuted}>A compact progress track announces its current value to accessibility tools.</text>
    <text color={theme.text}>{`Upload: ${progress}%`}</text>
    <Progress accessibleName="Upload progress" value={progress} width={280} />
    <Button onClick={() => setProgress((current) => current + 15 > 100 ? 0 : current + 15)}>Advance upload</Button>
    <Progress accessibleName="Processing progress" value={null} width={280} playing={playing} />
    <Button variant="outline" onClick={() => setPlaying((current) => !current)}>
      {playing ? 'Pause sweep' : 'Resume sweep'}
    </Button>
  </column>
}
