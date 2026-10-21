#!/usr/bin/env python3
"""Writes lctest.ttf, a tiny TrueType font drawn for LaserCraft's tests.

Own work, GPL-3.0-or-later like the rest of the repository. It covers space
and A C H I L O T, uses quadratic curves (O), has counters (A, O) and one
kern pair (L, T). Built from the OpenType specification's table layouts.
The output is deterministic (no timestamps).

    python3 examples/fixtures/make_lctest_font.py examples/fixtures/lctest.ttf
"""

import struct
import sys

UPM = 1000

# name: (advance, contours); a contour is a list of (x, y, on_curve).
def poly(*pts):
    return [(x, y, True) for x, y in pts]


GLYPHS = [
    (".notdef", 500, [poly((50, 0), (50, 700), (450, 700), (450, 0)),
                      poly((100, 50), (400, 50), (400, 650), (100, 650))]),
    ("space", 250, []),
    ("A", 600, [poly((50, 0), (250, 700), (350, 700), (550, 0), (450, 0),
                     (400, 200), (200, 200), (150, 0)),
                poly((230, 300), (300, 560), (370, 300))]),
    ("C", 600, [poly((50, 0), (50, 700), (550, 700), (550, 600), (150, 600),
                     (150, 100), (550, 100), (550, 0))]),
    ("H", 600, [poly((50, 0), (50, 700), (150, 700), (150, 400), (450, 400),
                     (450, 700), (550, 700), (550, 0), (450, 0), (450, 300),
                     (150, 300), (150, 0))]),
    ("I", 200, [poly((50, 0), (50, 700), (150, 700), (150, 0))]),
    ("L", 500, [poly((50, 0), (50, 700), (150, 700), (150, 100), (450, 100),
                     (450, 0))]),
    ("O", 600, [[(300, 0, True), (50, 0, False), (50, 350, True),
                 (50, 700, False), (300, 700, True), (550, 700, False),
                 (550, 350, True), (550, 0, False)],
                [(300, 100, True), (450, 100, False), (450, 350, True),
                 (450, 600, False), (300, 600, True), (150, 600, False),
                 (150, 350, True), (150, 100, False)]]),
    ("T", 600, [poly((250, 0), (250, 600), (50, 600), (50, 700), (550, 700),
                     (550, 600), (350, 600), (350, 0))]),
]
CHARS = {" ": 1, "A": 2, "C": 3, "H": 4, "I": 5, "L": 6, "O": 7, "T": 8}
KERN = [(6, 8, -100)]  # L, T


def glyph_data(contours):
    if not contours:
        return b""
    pts = [p for c in contours for p in c]
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    out = struct.pack(">hhhhh", len(contours), min(xs), min(ys), max(xs), max(ys))
    end = -1
    for c in contours:
        end += len(c)
        out += struct.pack(">H", end)
    out += struct.pack(">H", 0)  # no instructions
    out += bytes(1 if on else 0 for _, _, on in pts)
    px = py = 0
    for x, _, _ in pts:
        out += struct.pack(">h", x - px)
        px = x
    for _, y, _ in pts:
        out += struct.pack(">h", y - py)
        py = y
    if len(out) % 2:
        out += b"\0"
    return out


def checksum(data):
    data += b"\0" * (-len(data) % 4)
    return sum(struct.unpack(">%dI" % (len(data) // 4), data)) & 0xFFFFFFFF


def cmap():
    segs = sorted((ord(c), g) for c, g in CHARS.items())
    count = len(segs) + 1
    ends = [c for c, _ in segs] + [0xFFFF]
    starts = [c for c, _ in segs] + [0xFFFF]
    deltas = [(g - c) & 0xFFFF for c, g in segs] + [1]
    search = 2 * 2 ** (count.bit_length() - 1)
    sub = struct.pack(">HHHHHHH", 4, 0, 0, count * 2, search,
                      count.bit_length() - 1, count * 2 - search)
    sub += struct.pack(">%dH" % count, *ends) + b"\0\0"
    sub += struct.pack(">%dH" % count, *starts)
    sub += struct.pack(">%dH" % count, *deltas)
    sub += struct.pack(">%dH" % count, *([0] * count))
    sub = sub[:2] + struct.pack(">H", len(sub)) + sub[4:]
    return struct.pack(">HHHHI", 0, 1, 3, 1, 12) + sub


def name():
    strings = [
        (0, "Copyright 2026 LaserCraft contributors. GPL-3.0-or-later."),
        (1, "LaserCraft Test"),
        (2, "Regular"),
        (4, "LaserCraft Test Regular"),
        (6, "LaserCraftTest-Regular"),
    ]
    records = b""
    data = b""
    for nid, text in strings:
        raw = text.encode("utf-16-be")
        records += struct.pack(">HHHHHH", 3, 1, 0x409, nid, len(raw), len(data))
        data += raw
    return struct.pack(">HHH", 0, len(strings), 6 + len(records)) + records + data


def build():
    glyf = b""
    loca = []
    for _, _, contours in GLYPHS:
        loca.append(len(glyf) // 2)
        glyf += glyph_data(contours)
    loca.append(len(glyf) // 2)
    all_pts = [p for _, _, cs in GLYPHS for c in cs for p in c]
    xmin = min(p[0] for p in all_pts)
    xmax = max(p[0] for p in all_pts)
    max_points = max(sum(len(c) for c in cs) for _, _, cs in GLYPHS)
    max_contours = max(len(cs) for _, _, cs in GLYPHS)
    n = len(GLYPHS)
    tables = {
        b"head": struct.pack(">IIIIHHqqhhhhHHhhh", 0x00010000, 0x00010000, 0,
                             0x5F0F3CF5, 0x000B, UPM, 0, 0, xmin, 0, xmax, 700,
                             0, 8, 2, 0, 0),
        b"hhea": struct.pack(">IhhhHhhhhhhhhhhhH", 0x00010000, 800, -200, 0,
                             600, 50, 0, 550, 1, 0, 0, 0, 0, 0, 0, 0, n),
        b"maxp": struct.pack(">IHHHHHHHHHHHHHH", 0x00010000, n, max_points,
                             max_contours, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0),
        b"OS/2": struct.pack(">HhHHH10hh10s4I4sHHHhhhHHII5H", 4, 500, 400, 5, 0,
                             *([0] * 10), 0, b"\0" * 10, 0, 0, 0, 0, b"NONE",
                             0x0040, 0x20, 0x54, 800, -200, 0, 800, 200, 1, 0,
                             500, 700, 0, 32, 0),
        b"hmtx": b"".join(
            struct.pack(">Hh", adv, min((p[0] for c in cs for p in c), default=0))
            for _, adv, cs in GLYPHS),
        b"loca": struct.pack(">%dH" % len(loca), *loca),
        b"glyf": glyf,
        b"cmap": cmap(),
        b"kern": struct.pack(">HH", 0, 1)
        + struct.pack(">HHHHHHH", 0, 14 + 6 * len(KERN), 0x0001, len(KERN), 6, 0, 0)
        + b"".join(struct.pack(">HHh", *k) for k in KERN),
        b"name": name(),
        b"post": struct.pack(">IIhhIIIII", 0x00030000, 0, -100, 50, 0, 0, 0, 0, 0),
    }
    tags = sorted(tables)
    count = len(tags)
    search = 16 * 2 ** (count.bit_length() - 1)
    header = struct.pack(">IHHHH", 0x00010000, count, search,
                         count.bit_length() - 1, count * 16 - search)
    offset = 12 + 16 * count
    directory = b""
    body = b""
    for tag in tags:
        data = tables[tag]
        directory += struct.pack(">4sIII", tag, checksum(data), offset + len(body), len(data))
        body += data + b"\0" * (-len(data) % 4)
    font = bytearray(header + directory + body)
    head_at = 12 + 16 * count + sum(
        len(tables[t]) + (-len(tables[t]) % 4) for t in tags[: tags.index(b"head")])
    adjust = (0xB1B0AFBA - checksum(bytes(font))) & 0xFFFFFFFF
    font[head_at + 8: head_at + 12] = struct.pack(">I", adjust)
    return bytes(font)


if __name__ == "__main__":
    with open(sys.argv[1], "wb") as f:
        f.write(build())
