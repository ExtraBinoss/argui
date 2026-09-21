# Coverage sprint log

The eight requested agents each handled one crate and did not compile or run
coverage. The parent compiled their tests, then measured a clean instrumented
target: 1,973 passed, 2 skipped. The final quality run measured 88.13% branch coverage.
Earlier per-crate deficits in several crates were caused by
stale duplicate LLVM coverage maps; the coverage script now fully cleans its
dedicated instrumented target before the final measurement.

| Crate | Fresh branch coverage | Other metrics | Agent | Status |
| --- | --- | --- | --- | --- |
| `argui-paint` | 68/72 = 94.44% | All ≥97.16% | `cover_paint` | Pass |
| `dsl-compiler` | 241/282 = 85.46% | All ≥85.10% | `cover_compiler` | Pass |
| `argui-testing` | 127/142 = 89.44% | All ≥90.74% | `cover_semantic` | Pass |
| `argui-cli` | 213/250 = 85.20% | All ≥89.96% | `cover_cli` | Pass |
| `dsl-ir` | 127/148 = 85.81% | All ≥93.73% | `cover_ir` | Pass |
| `argui-runtime` | 670/786 = 85.24% | All ≥88.20% | `cover_runtime` | Pass |
| `argui-schema` | 272/296 = 91.89% | All ≥96.81% | `cover_schema` | Pass |
| `dsl-runtime` | 236/274 = 86.13% | All ≥85.88% | `cover_dsl_runtime` | Pass |

`dsl-semantic`, separately, passes all metrics (85.65% branches). No threshold,
exclusion, or production behavior was changed to improve these numbers. The
parent alone runs Cargo, hidden-display native checks, and LLVM coverage.
