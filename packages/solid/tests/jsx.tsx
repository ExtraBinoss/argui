/** @jsxImportSource @argui/solid */
import type { JSX } from '@argui/solid/jsx-runtime'

/** Solid consumes the same generated native layout contract as React. */
export const layoutFixture: JSX.Element = <column gap={12} padding={{ top: 8, start: 16 }}>
  <text>Bonjour</text>
  <grid gridColumns={[{ repeat: { count: 'autoFit', tracks: [{ minmax: { min: 160, max: { fr: 1 } } }] } }]} />
</column>

// @ts-expect-error framework key does not name a native node
export const invalidNativeKey: JSX.Element = <column nativeKey="old" />
// @ts-expect-error text children and an explicit text prop are exclusive
export const invalidText: JSX.Element = <text text="A">B</text>
// @ts-expect-error fill has no defined v2 dimension meaning
export const invalidFill: JSX.Element = <column width="fill" />
