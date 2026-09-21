# ADR 0007: Renderer-independent WGSL validation

## Decision

`argui-shader` owns custom-effect ABI wrapping, Naga parsing/validation,
entry-point checks, parameter metadata validation, source hashing, and
`ShaderSourceMap`. It has no WGPU dependency.

Both build compilation and `argui-render` consume the same validated shader
artifact. Diagnostics translate wrapped-source byte ranges back to the
original WGSL file, line, and column; generated ABI ranges are clearly marked
as framework errors.
