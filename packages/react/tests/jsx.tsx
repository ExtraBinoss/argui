/** @jsxImportSource @argui/react */
import type { JSX } from '@argui/react/jsx-runtime'

/** Compile-time fixture proving the native schema drives React JSX properties. */
export const fixture: JSX.Element = <column gap={12}><text text="Hello" /></column>

/** A native viewport uses typed controls while its frames stay in Rust. */
export const viewportFixture: JSX.Element = <gpuCanvas canvas_id={1} width="100%" height={240} resolution_scale={1.5} alt="Preview" />
