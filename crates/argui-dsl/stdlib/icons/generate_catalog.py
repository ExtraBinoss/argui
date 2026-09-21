"""Regenerate catalog.txt from the pinned icondata_tb 0.1.0 src/lib.rs.

Usage: python3 generate_catalog.py /path/to/icondata_tb-0.1.0/src/lib.rs
The upstream icondata_tb crate and Tabler glyphs are MIT licensed.
"""

import re
import sys
from pathlib import Path


def main() -> None:
    source = Path(sys.argv[1]).read_text(encoding="utf-8")
    symbols = re.findall(r"^pub static Tb([A-Za-z0-9]+)(Outline|Filled):", source, re.M)
    names = sorted(name + ("Filled" if style == "Filled" else "") for name, style in symbols)
    assert len(names) == len(set(names)) == 5944, "unexpected upstream Tabler catalog"
    Path(__file__).with_name("catalog.txt").write_text("\n".join(names) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
