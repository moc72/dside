#!/usr/bin/env python3
"""Turn a saved names table (HTML fragment) into our own table.

The pages the user saves carry one `<tr>` per name with the columns
Name | Herkunft | Bedeutung | Beliebtheit | Trend. Popularity is not text — it is the
width of a coloured bar in an inline SVG, scaled to 50. Trend is drawn on a `<canvas>`,
so its data is not in the HTML at all and stays empty here.

Usage:
    python3 scripts/namen_import.py src/assets/namen.html [--append]

Without `--append` the output files are rewritten; with it, rows are added to what is
already there and exact duplicates are dropped. `--report` only prints what it found.
"""
from __future__ import annotations

import collections
import html
import io
import json
import pathlib
import re
import sys

COLUMNS = ["name", "origin", "meaning", "popularity"]
BAR_SCALE = 50.0


def cells(row: str) -> list[str]:
    return re.findall(r"<td[^>]*>(.*?)</td>", row, flags=re.S)


def text_of(cell: str) -> str:
    cell = re.sub(r"<svg.*?</svg>", " ", cell, flags=re.S)
    cell = re.sub(r"<canvas.*?</canvas>", " ", cell, flags=re.S)
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", cell))).strip()


def popularity(cell: str) -> float | None:
    widths = [float(w) for w in re.findall(r'<rect[^>]*width="([0-9.]+)"', cell)]
    if not widths:
        return None
    # First rectangle is the grey track (always full), the coloured one holds the value.
    value = next((w for w in widths if w < BAR_SCALE), widths[0])
    return round(value / BAR_SCALE * 100, 1)


def parse(path: pathlib.Path) -> list[dict]:
    source = path.read_text(encoding="utf-8")
    table = []
    for row in re.findall(r"<tr>(.*?)</tr>", source, flags=re.S):
        columns = cells(row)
        if len(columns) < 4:
            continue
        values = [text_of(column) for column in columns]
        name, origin, meaning = values[1], values[2], values[3]
        if not name or not meaning:
            continue
        table.append(
            {
                "name": name,
                "origin": origin,
                "meaning": meaning,
                "popularity": popularity(columns[4]) if len(columns) > 4 else None,
            }
        )
    return table


def write_table(directory: pathlib.Path, rows: list[dict]) -> None:
    with io.open(directory / "namen.tsv", "w", encoding="utf-8") as out:
        out.write("Name\tHerkunft\tBedeutung\tBeliebtheit\n")
        for entry in rows:
            pop = "" if entry["popularity"] is None else str(entry["popularity"])
            out.write(f"{entry['name']}\t{entry['origin']}\t{entry['meaning']}\t{pop}\n")

    with io.open(directory / "namen.json", "w", encoding="utf-8") as out:
        json.dump(rows, out, ensure_ascii=False, indent=1)
        out.write("\n")


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2

    source = pathlib.Path(argv[1])
    if not source.exists():
        print(f"not found: {source}")
        return 1

    rows = parse(source)
    directory = source.parent

    if "--append" and (directory / "namen.json").exists():
        existing = json.loads((directory / "namen.json").read_text(encoding="utf-8"))
        known = {tuple(entry.get(key) for key in COLUMNS) for entry in existing}
        added = [entry for entry in rows if tuple(entry.get(key) for key in COLUMNS) not in known]
        rows = existing + added
        print(f"appended {len(added)} rows, {len(rows)} in total")

    names = [entry["name"] for entry in rows]
    duplicates = {name for name, count in collections.Counter(names).items() if count > 1}
    meanings = collections.Counter(entry["meaning"] for entry in rows)

    print(f"{source.name}: {len(rows)} rows, {len(set(names))} distinct names")
    print(f"  duplicated names kept: {len(duplicates)}")
    print(f"  distinct meanings: {len(meanings)} — top: {meanings.most_common(5)}")
    print(f"  without popularity: {sum(1 for entry in rows if entry['popularity'] is None)}")

    if "--report" not in argv:
        write_table(directory, rows)
        print(f"  wrote {directory / 'namen.tsv'} and {directory / 'namen.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
