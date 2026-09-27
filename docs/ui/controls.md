# Widgets and controls

`@argui/widgets/solid` and `@argui/widgets/react` export the same widget set.
They render Argui native primitives through their respective adapters; they are
not HTML components. Mount one `ThemeProvider` above them as shown in
[Theme](theme.md). Imports below use Solid. For React, change the import to
`@argui/widgets/react`, use React state, and add
`/** @jsxImportSource @argui/react */` to the TSX file.

The workspace package exports all widgets in the table. The gallery has
individual pages for twelve of them; `ScrollShadow` appears in its scrollbar
example instead. The `argui add` CLI registry is a separate source-copy
catalogue and currently lists only `button`, `button-group`, `input-field`,
`popover`, `scroll-shadow`, `select`, `tooltip`, and `virtual-list`. A gallery
page or workspace export does not imply a component is available through
`argui add`. The CLI only installs from a verified release registry and source
files at the application's exact version tag; an embedded development SDK
rejects that command.

| Widget | Purpose and essential props |
| --- | --- |
| `Button` | Action with required `onClick`; `variant`, `size`, `disabled`, `pressed`, `contentAlign` and `pressAnimation` control its presentation. |
| `ButtonGroup`, `ButtonGroupSeparator`, `ButtonGroupText` | Joined actions or inputs in one named group. |
| `InputField` | Single-line native editor; `label` or `accessibleName`, `value`/`onValueChange` or `defaultValue`, `type="text" \| "search" \| "password"`. |
| `Select` | Single choice from unique `{ value, label, disabled? }` options; `label` is required. |
| `Checkbox`, `Switch` | Boolean controls with required `accessibleName`; optional visible `label`. `Switch` also has `size="sm"`. |
| `Tabs` | One selected `{ value, label, content, disabled? }` panel; `accessibleName` and `items` are required. |
| `Slider` | Horizontal, single-thumb numeric range; `accessibleName`, optional `min`, `max`, `step`. |
| `Progress` | Read-only progress; `value={null}` means indeterminate, and `playing={false}` pauses its native sweep. |
| `Popover`, `Tooltip` | Anchored floating content; see [Interaction](interaction.md). |
| `ScrollShadow`, `VirtualList` | Native scroll-edge fading and a bounded mounted list; see [Scrolling](scroll.md). |

Every widget accepts the outer layout props in `WidgetLayoutProps`, except
where its type narrows them. `width` and `height` are preferred sizes in logical
pixels, `"auto"`, or a percentage; `grow` takes remaining flex space. Native
`id` is optional and is useful for anchors, accessibility relations, and tests.
Framework `key` is a separate reconciliation identity.

`Checkbox` accepts an optional `checkedIcon` asset reference. The gallery passes
its Tabler `check.svg` to all checkbox examples; an application can supply its
own SVG from its asset pack. See [application icons](icons.md) for asset setup.

## State ownership

`Checkbox`, `Switch`, `Tabs`, and `Slider` use `defaultValue` for local state or
`value` with `onValueChange` for controlled state. A supplied `value` without
the callback is read-only. `InputField` requires `onValueChange` with a
controlled `value`, unless `readOnly` is explicitly true. Its `defaultValue`
lets the native editor own subsequent edits. `Select` with `value` but no
`onValueChange` disables selection. Its option values must be unique.

This Solid example keeps editable state in the component:

```tsx solid
import { createSignal } from '@argui/solid'
import {
  Button, Checkbox, InputField, Progress, Select, Slider, Switch,
  type SelectOption,
} from '@argui/widgets/solid'

const languages: SelectOption[] = [
  { value: 'rust', label: 'Rust' },
  { value: 'typescript', label: 'TypeScript' },
]

export function Preferences() {
  const [name, setName] = createSignal('Ada')
  const [language, setLanguage] = createSignal('rust')
  const [updates, setUpdates] = createSignal(true)
  const [sounds, setSounds] = createSignal(false)
  const [volume, setVolume] = createSignal(40)

  return <column width="100%" gap={12}>
    <InputField label="Name" value={name()} onValueChange={setName} required />
    <Select label="Language" options={languages} value={language()}
      onValueChange={setLanguage} />
    <Checkbox accessibleName="Product updates" label="Product updates"
      value={updates()} onValueChange={setUpdates} />
    <Switch accessibleName="Sounds" label="Sounds"
      value={sounds()} onValueChange={setSounds} />
    <Slider accessibleName="Volume" value={volume()} onValueChange={setVolume}
      min={0} max={100} step={5} width={280} />
    <Progress accessibleName="Profile completion" value={60} width={280} />
    <Button onClick={() => setName('')}>Clear name</Button>
  </column>
}
```

The React adapter uses the same control props and React state:

```tsx react
/** @jsxImportSource @argui/react */
import { useState } from 'react'
import { Button, InputField } from '@argui/widgets/react'

export function SearchForm(props: { onSearch: (query: string) => void }) {
  const [query, setQuery] = useState('')
  return <row gap={8}>
    <InputField accessibleName="Search terms" type="search" width={220}
      value={query} onValueChange={setQuery} onSubmit={props.onSearch} />
    <Button onClick={() => props.onSearch(query)}>Search</Button>
  </row>
}
```

`InputField` accepts `placeholder`, `description`, `disabled`, `invalid`,
`required`, `onSubmit`, and application-owned `leading`/`trailing` content.
`type="password"` adds a Show/Hide action while the native editor retains its
buffer. The visible `label` also names the editor; use `accessibleName` when the
label is elsewhere. `description` is visible help text and is also passed to
the native editor as its accessible description. The focus border follows the
editor's native focus state. `type="search"` changes the native editor's search
semantics; no `email` type or built-in email validator is defined. The theme's
`inputLineHeight` aligns its glyphs with adjacent icon slots, and
`inputGroupHeight` applies when the field sits in a `ButtonGroup`.

`Select` composes a combobox trigger and a popup listbox. It supports
`variant="default"` and `variant="shadcn"`; the latter puts its group label and
a selectable placeholder in the popup. Disabled options stay visible but
cannot be selected. `leading` and `trailing` accept application content; a
provided trailing chevron rotates when the popup opens. `defaultOpen` starts
with locally managed popup state. For controlled popup state, pair `open` with
`onOpenChange`. Its width also sizes the option popup, and the popup scrolls
when its measured rows exceed the theme's `selectMaxPopupHeight`.

## Buttons, groups, and tabs

Button variants are `default`, `outline`, `secondary`, `ghost`, `destructive`,
and `link`. `link` is only a visual variant: supply the navigation behavior in
`onClick`. Sizes are `default`, `xs`, `sm`, `lg`, and their `icon` forms. Give
icon-only buttons an `accessibleName`; the widget has no icon-pack dependency.
`pressed` exposes a persistent toggle state to accessibility. It does not own
that state. `pressAnimation={false}` disables the native press scale.

```tsx solid
import { createSignal, useTheme } from '@argui/solid'
import {
  Button, ButtonGroup, ButtonGroupSeparator, Tabs, type WidgetTheme,
} from '@argui/widgets/solid'

export function DocumentActions() {
  const [result, setResult] = createSignal('Choose an action')
  const theme = useTheme<WidgetTheme>()
  const items = [
    { value: 'details', label: 'Details', content: <text color={theme().text}>Document details</text> },
    { value: 'history', label: 'History', content: <text color={theme().text}>Edit history</text> },
  ]

  return <column gap={12}>
    <ButtonGroup accessibleName="Document actions">
      <Button variant="outline" onClick={() => setResult('Archived')}>Archive</Button>
      <ButtonGroupSeparator />
      <Button variant="outline" onClick={() => setResult('Shared')}>Share</Button>
    </ButtonGroup>
    <text color={theme().text}>{result()}</text>
    <Tabs accessibleName="Document sections" items={items} />
  </column>
}
```

`ButtonGroup` has horizontal and vertical orientations and accepts
`directionScope="rtl"`. Its `accessibleName` names the group; each child
remains its own keyboard target. Group `Button` and `InputField` instances
inside its shared clipped border. `ButtonGroupSeparator` follows the nearest
group orientation unless given its own `orientation`. `ButtonGroupText` accepts
plain children or `render` for custom content. The group is intrinsically
sized in a column; use `width` or `alignSelf="stretch"` when it should fill
available width. `Tabs` supports `variant="line"` and vertical
orientation. Arrow keys choose an enabled tab along the selected axis; Home
and End choose the first and last enabled tabs. Keep each tab's `value` stable.

## Native behavior and limits

`Slider` defaults to 0–100 with step 1, clamps and snaps values, and supports
track click, drag, arrow keys, Page Up/Down, Home/End, and semantic value
actions. It is horizontal and has one thumb; the package exposes no vertical
or multi-thumb variant. `Progress` clamps a numeric value to 0–`max` (default
100). For an indeterminate sweep sized to the entire track, give it a numeric
`width`; without one it uses a fixed sweep distance. The native reduced-motion
preference stops the sweep's frame requests.

The adapters keep visible paint, keyboard focus, and accessibility on native
nodes. A TypeScript check verifies props, while a host mount verifies the
schema and platform behavior. The [Solid gallery](../../apps/gallery/src/solid/gallery.tsx)
and [React gallery](../../apps/gallery/src/react/gallery.tsx) show the widgets
mounted in the application.
