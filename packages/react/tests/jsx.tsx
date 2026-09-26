/** @jsxImportSource @argui/react */
import type { JSX } from '@argui/react/jsx-runtime'

/** Compile-time fixture proving the native schema drives React JSX properties. */
export const fixture: JSX.Element = <column gap={12}><text>Hello</text></column>

/** A native viewport uses typed controls while its frames stay in Rust. */
export const viewportFixture: JSX.Element = <gpuCanvas canvasId={1} width="100%" height={240} resolutionScale={1.5} alt="Preview" />

/** Responsive grid rules are typed and evaluated by the native layout engine. */
export const responsiveFixture: JSX.Element = <grid containerScope="cards" gridColumns={[{ fr: 1 }]}
  containerRules={[{ scope: 'cards', when: { minWidth: 480 }, style: {
    gridColumns: [{ repeat: { count: 'autoFit', tracks: [{ minmax: { min: 160, max: { fr: 1 } } }] } }],
  } }]} />

/** Invalid native options fail during TypeScript checking. */
// @ts-expect-error misspelled closed placement option
export const invalidPlacement: JSX.Element = <popupWindow placement="bottom_strat" />
// @ts-expect-error ambiguous legacy sizing has no v2 meaning
export const invalidFill: JSX.Element = <column width="fill" />
// @ts-expect-error framework key cannot be passed through a nativeKey alias
export const invalidNativeKey: JSX.Element = <column nativeKey="old" />
// @ts-expect-error text content has one source
export const invalidText: JSX.Element = <text text="A">B</text>
// @ts-expect-error grid tracks are typed values, not a CSS string parser
export const invalidGrid: JSX.Element = <grid gridColumns="repeat(auto-fit, minmax(160px, 1fr))" />
// @ts-expect-error a closed alignment option cannot be misspelled
export const invalidAlignment: JSX.Element = <row alignItems="centerr" />
