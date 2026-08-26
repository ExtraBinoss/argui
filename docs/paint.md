# Paint model

`argui-paint` describes pixels without knowing WGPU, widgets, themes, or a DSL.
`DisplayList` keeps `Quad` and text references in exact paint order. The renderer
merges only adjacent commands of the same pipeline.

Current quad data includes a solid `Fill`, background color, border color and
width per side, radius per corner, primitive opacity, bounds, and a resolved clip
rectangle. `Fill` is an enum so gradients can be added without changing the UI
tree's background type.

`Element::paint_opacity` affects that element's quad only. Correct group opacity,
blur, shadows, rounded descendant clipping, and backdrop filters require layers
or clip masks and are deliberately deferred to the layers/effects milestone.
There are no hidden allocations or off-screen textures for ordinary quads.

The staged API and renderer work for these features is specified in the
[effects implementation plan](effects.md). Paint will describe layer semantics;
only `argui-render` will own render passes, texture pools, pipelines, and WGSL.
