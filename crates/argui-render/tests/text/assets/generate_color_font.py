"""Rebuild the original Argui color test font with fontTools 4.65.0.

The two glyphs are original geometric outlines, licensed with this repository.
fontTools is a generation-only tool; runtime tests read the committed font.
"""
from pathlib import Path

from fontTools.colorLib.builder import buildCOLR, buildCPAL
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen


def outline(points=()):
    """Return a closed TrueType glyph built from the original polygon points."""
    pen = TTGlyphPen(None)
    if points:
        pen.moveTo(points[0])
        for point in points[1:]:
            pen.lineTo(point)
        pen.closePath()
    return pen.glyph()


font = FontBuilder(1000, isTTF=True)
font.setupGlyphOrder([".notdef", "A", "B", "polygon"])
font.setupCharacterMap({ord("A"): "A", ord("B"): "B"})
font.setupGlyf({
    ".notdef": outline(),
    "A": outline(),
    "B": outline(),
    "polygon": outline([(100, 100), (900, 100), (740, 900), (100, 900)]),
})
font.setupHorizontalMetrics({name: (1000, 0) for name in font.font.getGlyphOrder()})
font.setupHorizontalHeader(ascent=1000, descent=0)
font.setupNameTable({
    "familyName": "Argui Color Test",
    "styleName": "Regular",
    "uniqueFontIdentifier": "Argui Color Test 1.0",
    "fullName": "Argui Color Test",
    "psName": "ArguiColorTest",
    "version": "Version 1.0",
})
font.setupOS2(sTypoAscender=1000, sTypoDescender=0, usWinAscent=1000, usWinDescent=0)
font.setupPost()
font.font["COLR"] = buildCOLR({"A": [("polygon", 0)], "B": [("polygon", 1)]}, version=0)
font.font["CPAL"] = buildCPAL([[(0.2, 0.4, 0.8, 1.0), (0.8, 0.2, 0.4, 0.5)]])
font.font["head"].created = 2082844800
font.font["head"].modified = 2082844800
font.font.recalcTimestamp = False
font.save(Path(__file__).with_name("test-color.ttf"))
