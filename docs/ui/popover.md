# Popover

`Popover` is the reference for nonmodal floating content in the Solid and
React widget adapters. The application supplies a trigger label and children;
the component owns its native anchor, popup identity, dismissal, and focus
restoration.

```tsx
<Popover trigger="Filters" contentWidth={320}>
  <InputField label="Query" defaultValue="" />
</Popover>
```

`width` controls the trigger's outer layout only. `contentWidth` controls the
floating surface and defaults to the theme's `overlayWidth`. The trigger keeps
its natural width when `width` is omitted. `leading` and `trailing` accept
application-owned icon or content elements. `accessibleLabel` overrides the
trigger text for the trigger and popup's accessible name. A caller can provide
`id` for tests or automation; the component otherwise creates one stable ID
for its lifetime.

Use `defaultOpen` for a locally managed initial state or pair `open` with
`onOpenChange` for a controlled state. TypeScript rejects `open` without the
callback and rejects combining `open` with `defaultOpen`.

```tsx
<Popover trigger="Filters" open={open()} onOpenChange={setOpen}>
  ...
</Popover>
```

The default popup does not trap focus or move it from the trigger. Outside
pointer and Escape dismiss it, and focus is restored to the trigger. Set
`initialFocus="first"` if its first focusable child should receive focus on
opening. A modal dialog needs a different focus-containment policy.

Blurred popovers use `overlaySurface` together with `overlayBlur`, so text has
a themed translucent backing. `opaque` uses the regular `surface` token.
`overlayPadding`, `overlayRadius`, `overlayBorderWidth`, and the
`overlayShadow*` tokens define the remaining surface style. The same tokens
can be reused by Select and future floating widgets.
