# InputField

`InputField` wraps the native `textInput` in Solid and React. The native editor
owns the text buffer, caret, selection, composition, and focus. The wrapper
reads colors and dimensions from the active widget theme. A native `id` is
optional; supply one only for an external relation or automation.

Use `value` with `onValueChange` for a controlled field. Use `defaultValue` for
native autonomous editing. To display a controlled value without allowing
changes, set `readOnly` explicitly. TypeScript rejects mixing `value` and
`defaultValue`, or passing `value` without a callback or `readOnly`.

```tsx
<InputField label="Name" value={name} onValueChange={setName} />
<InputField accessibleName="Search" type="search" defaultValue="" />
<InputField label="Reference" value="A-102" readOnly />
```

`label` gives the field a visible label and an accessible name. If the design
already shows a label elsewhere, use `accessibleName`. `description` renders
help text and passes it to the native editor for accessibility. `required`,
`invalid`, and `disabled` expose the corresponding native states. The focus
border follows the editor's focus in the native tree, without JavaScript state.

`leading` and `trailing` accept application-owned icons or other TSX content.
`type="password"` adds a small Show/Hide control with an accessible name;
the native editor keeps the text while its privacy mode changes.

The theme provides `inputHeight`, `inputGroupHeight`, `inputLineHeight`, and
`fieldLabelSize`. `inputLineHeight` should match the text line height so glyphs
align with adjacent icons. The grouped height is used inside `ButtonGroup`.
