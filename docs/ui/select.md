# Select

`Select` composes a native combobox trigger and a popup listbox. It accepts an
array of options with unique `value` strings, plus a visible `label`. Its
native `id` is optional and generated when the caller does not supply one.

```tsx
<Select label="Language" options={languages} defaultValue="rust" />
<Select label="Language" options={languages}
  value={language} onValueChange={setLanguage} />
```

`defaultValue` starts a locally managed selection. With `value`, the caller
owns selection; omitting `onValueChange` leaves that value read-only. For a
controlled popup, pair `open` with `onOpenChange`. TypeScript rejects mixed
controlled and default state for each value and popup state independently.
`leading` and `trailing` accept app-provided content. Disabled options remain
visible but cannot be chosen. The trigger exposes its value and active option
to assistive technology.

The theme sets `selectWidth`, `selectCompactHeight`, `selectRowHeight`,
`selectCompactRowHeight`, and `selectMaxPopupHeight`. `inputHeight` and
`fieldLabelSize` are shared with InputField. Override the outer `width` for a
different trigger and popup width.
