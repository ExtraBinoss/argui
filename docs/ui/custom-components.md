# Custom components and state

Argui's TSX primitives are lowercase: `rectangle` paints a surface,
`row` and `column` arrange children, and `text` draws content. A custom
component is an ordinary Solid or React function that returns those primitives.
The adapter sends the resulting tree to the Rust host, which owns layout,
painting, input, and retained element identity. You do not need to add a Rust
primitive for every reusable card or application component.

## Compose a card from primitives

This Solid component accepts data through props. Two instances reuse the same
tree structure with different values:

```tsx solid
type MetricCardProps = { id: string; label: string; value: string; accent: string }

export function MetricCard(props: MetricCardProps) {
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

export function CustomElementsExample() {
  return <row gap={12} wrap={true}>
    <MetricCard id="metric-queue" label="Queue" value="Healthy" accent="#16a34a" />
    <MetricCard id="metric-latency" label="Latency" value="24 ms" accent="#8b5cf6" />
  </row>
}
```

The live Solid example below runs on the Rust WebAssembly host. Each card is
one custom component, and each nested primitive becomes a retained native
element.

<!-- argui-example:custom-elements -->

`rectangle` supplies paint, radius, and padding. `column` stacks the label
and value; `row` places the dot beside the label. `text` needs an explicit
color because raw text does not automatically inherit widget theme tokens.
The `id` prop is an optional native identity for references and tests; when
mapping a changing list in Solid or React, also give each component a stable
framework `key`.

In Rust, the corresponding surface is an `Element::container` or any layout
element with paint applied. There is no separate `Element::rectangle`
constructor:

```rust
use argui_ui::{Color, Element, length};

fn metric_card(label: &str, value: &str) -> Element {
    Element::container([
        Element::column([
            Element::row([Element::text(label)]).gap(8.0),
            Element::text(value),
        ])
        .gap(10.0),
    ])
    .width(length(220.0))
    .background(Color::WHITE)
}
```

## Add state with a separate counter

Keep state in the framework component. An Argui `Button` handles native
pointer, keyboard, focus, and accessibility behavior; its callback updates the
signal. Solid then sends only the changed text to the retained host:

```tsx solid
import { createSignal } from '@argui/solid'
import { Button } from '@argui/widgets/solid'

export function CounterExample() {
  const [count, setCount] = createSignal(0)
  return <column width="100%" gap={16}>
    <text color="#18181b" fontSize={22}>Stateful counter</text>
    <text color="#57534e">Click the native button to update Solid state.</text>
    <row gap={16} alignItems="center">
      <Button id="example-counter-increment" onClick={() => setCount(count() + 1)}>Increment</Button>
      <text id="example-counter-value" color="#18181b" fontSize={18}>{`Count: ${count()}`}</text>
    </row>
  </column>
}
```

This is a second, independent WebAssembly example. Click **Increment** and
watch the count change without remounting the scene.

<!-- argui-example:counter -->

React uses the same primitives and native host; its state setter uses the
functional form:

```tsx react
/** @jsxImportSource @argui/react */
import { useState } from 'react'
import { Button } from '@argui/widgets/react'

export function CounterExample() {
  const [count, setCount] = useState(0)
  return <row gap={16} alignItems="center">
    <Button onClick={() => setCount(current => current + 1)}>Increment</Button>
    <text color="#18181b">{`Count: ${count}`}</text>
  </row>
}
```

For sizing and flex behavior, read [layout](layout.md). For paint and clipping,
read [styling](styling.md). For widget theme tokens and interactive controls,
read [theme](theme.md) and [controls](controls.md). The [Solid and React host
contract](../solid-react-native.md) explains reconciliation and native IDs.
The browser examples require WebGPU; the same components also run in the
desktop gallery.
