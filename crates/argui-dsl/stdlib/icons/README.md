# Tabler catalog

`catalog.txt` records all 5,944 exported outline and filled symbols from the
pinned `icondata_tb = 0.1.0` crate. Regenerate it using `generate_catalog.py`
with that crate's `src/lib.rs` as its argument. Unsuffixed names select the
outline variant; `Filled` names select filled variants. `Spinner` is the only
Argui alias and maps to `Loader2`.

The original Tabler icons and the `icondata_tb` package are MIT licensed.
This manifest contains symbol names only; icon geometry comes from the pinned
dependency at compile time. The AOT generator emits only reachable SVG bytes
into the application, not the entire catalog.
