# ButtonGroup

`ButtonGroup` joins related Argui controls into one bordered surface. Its
composition follows the [shadcn/ui Button Group API](https://ui.shadcn.com/docs/components/base/button-group):
`ButtonGroup`, `ButtonGroupSeparator`, and `ButtonGroupText` work with ordinary
`Button` and `InputField` components in Solid and React. The group exposes a
native `group` role and requires an accessible name. Each child keeps its own
focus target, keyboard activation, and action.

```tsx
<ButtonGroup accessibleName="Document actions">
  <Button variant="ghost" onClick={archive}>Archive</Button>
  <ButtonGroupSeparator />
  <Button variant="ghost" onClick={report}>Report</Button>
</ButtonGroup>
```

The default orientation is horizontal. Set `orientation="vertical"` for a
stack, and `directionScope="rtl"` for a right-to-left group. A separator is
perpendicular to its nearest group by default; its own `orientation` can
override that. `ButtonGroupText` accepts plain children or `render` for custom
content. The widgets accept app-provided icons as children or input slots and
have no icon-pack dependency.
The group sizes to its contents by default inside a column and clips its
children to the same theme radius as other controls. Use `width` or
`alignSelf="stretch"` when it should fill the available width.

```tsx
<ButtonGroup accessibleName="Search documents">
  <InputField accessibleName="Search term" type="search" width={220}
    value={query} onValueChange={setQuery} />
  <ButtonGroupSeparator />
  <Button variant="ghost" onClick={search}>Search</Button>
</ButtonGroup>
```

Use `ButtonGroup` for related actions. A two-state choice such as the gallery
theme selector can also use buttons with explicit `pressed` semantics and
state-dependent paint. See the Solid and React `ButtonGroup` gallery pages for
working split actions, search, vertical layout, and RTL examples.
