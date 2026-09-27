/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button } from '@argui/widgets/react'

type MetricCardProps = { id: string; label: string; value: string; accent: string }

/** Composes native layout and paint primitives into one reusable card. */
export function MetricCard(props: MetricCardProps): ReactElement {
  return <rectangle id={props.id} width={220} padding={16} radii={12} background="#f4f4f5">
    <column gap={10}>
      <row gap={8} alignItems="center">
        <rectangle width={8} height={8} radii={4} background={props.accent} />
        <text color="#57534e" fontSize={13}>{props.label}</text>
      </row>
      <text color="#18181b" fontSize={24}>{props.value}</text>
    </column>
  </rectangle>
}

/** Shows two instances of the same custom primitive composition. */
export function CustomElementsExample(): ReactElement {
  return <column width="100%" gap={16}>
    <text color="#18181b" fontSize={22}>Custom element</text>
    <text color="#57534e">The same component receives different props.</text>
    <row gap={12} wrap={true}>
      <MetricCard id="metric-queue" label="Queue" value="Healthy" accent="#16a34a" />
      <MetricCard id="metric-latency" label="Latency" value="24 ms" accent="#8b5cf6" />
    </row>
  </column>
}

/** Shows React state changing a retained text node after a native Button click. */
export function CounterExample(): ReactElement {
  const [count, setCount] = useState(0)
  return <column width="100%" gap={16}>
    <text color="#18181b" fontSize={22}>Stateful counter</text>
    <text color="#57534e">Click the native button to update React state.</text>
    <row gap={16} alignItems="center">
      <Button id="example-counter-increment" onClick={() => setCount(current => current + 1)}>Increment</Button>
      <text id="example-counter-value" color="#18181b" fontSize={18}>{`Count: ${count}`}</text>
    </row>
  </column>
}
