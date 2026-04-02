#!/usr/bin/env python3
import argparse
import csv
import math
from pathlib import Path


def load_rows(path: Path):
    rows = []
    with path.open("r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                t = int(row["threads"])
                med = float(row["max_abs_delta_median"])
                p10 = float(row["max_abs_delta_p10"])
                p90 = float(row["max_abs_delta_p90"])
                b = float(row["max_abs_delta_bankruptcy_rate"])
                s = float(row["max_abs_delta_success_rate"])
            except (ValueError, KeyError):
                continue
            rows.append((t, med, p10, p90, b, s))
    rows.sort(key=lambda x: x[0])
    return rows


def write_svg(rows, out: Path):
    width, height = 980, 560
    ml, mr, mt, mb = 100, 30, 60, 80
    pw, ph = width - ml - mr, height - mt - mb

    xs = [r[0] for r in rows]
    x_min, x_max = min(xs), max(xs)
    if x_min == x_max:
        x_min -= 1
        x_max += 1

    vals = []
    for _, med, p10, p90, b, s in rows:
        vals.extend([med, p10, p90, b, s])
    y_min = 1e-12
    y_max = max(max(vals), y_min) * 10

    def sx(x):
        return ml + (x - x_min) / (x_max - x_min) * pw

    def sy(y):
        y = max(y, y_min)
        lo, hi = math.log10(y_min), math.log10(y_max)
        ly = math.log10(y)
        return mt + (1 - (ly - lo) / (hi - lo)) * ph

    series = [
        ("median", 1, "#2563eb"),
        ("p10", 2, "#16a34a"),
        ("p90", 3, "#9333ea"),
        ("bankruptcy", 4, "#dc2626"),
        ("success", 5, "#ea580c"),
    ]

    lines = []
    lines.append(f"<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>")
    lines.append("<rect width='100%' height='100%' fill='white' />")
    lines.append("<text x='490' y='30' text-anchor='middle' font-size='22' font-family='monospace'>MT Accuracy Drift vs Serial (log scale)</text>")

    lo, hi = math.log10(y_min), math.log10(y_max)
    for i in range(6):
        lv = lo + (hi - lo) * i / 5
        v = 10 ** lv
        y = mt + (1 - i / 5) * ph
        lines.append(f"<line x1='{ml}' y1='{y:.1f}' x2='{width-mr}' y2='{y:.1f}' stroke='#e5e7eb' />")
        lines.append(f"<text x='{ml-10}' y='{y+4:.1f}' text-anchor='end' font-size='12' font-family='monospace'>{v:.2e}</text>")

    for x in xs:
        xx = sx(x)
        lines.append(f"<line x1='{xx:.1f}' y1='{mt}' x2='{xx:.1f}' y2='{height-mb}' stroke='#f3f4f6' />")
        lines.append(f"<text x='{xx:.1f}' y='{height-mb+22}' text-anchor='middle' font-size='12' font-family='monospace'>{x}</text>")

    lines.append(f"<line x1='{ml}' y1='{height-mb}' x2='{width-mr}' y2='{height-mb}' stroke='#111827' />")
    lines.append(f"<line x1='{ml}' y1='{mt}' x2='{ml}' y2='{height-mb}' stroke='#111827' />")

    for name, idx, color in series:
        pts = " ".join(f"{sx(r[0]):.1f},{sy(r[idx]):.1f}" for r in rows)
        lines.append(f"<polyline points='{pts}' fill='none' stroke='{color}' stroke-width='2.5' />")
        for r in rows:
            x, y = sx(r[0]), sy(r[idx])
            lines.append(f"<circle cx='{x:.1f}' cy='{y:.1f}' r='3.5' fill='{color}' />")

    lx = width - mr - 220
    ly = mt + 10
    for i, (name, _, color) in enumerate(series):
        y = ly + i * 18
        lines.append(f"<line x1='{lx}' y1='{y}' x2='{lx+22}' y2='{y}' stroke='{color}' stroke-width='3' />")
        lines.append(f"<text x='{lx+30}' y='{y+4}' font-size='12' font-family='monospace'>{name}</text>")

    lines.append("</svg>")
    out.write_text("\n".join(lines), encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Plot MT accuracy drift over threads")
    p.add_argument("accuracy_csv")
    p.add_argument("--output", required=True)
    args = p.parse_args()

    rows = load_rows(Path(args.accuracy_csv))
    if not rows:
        raise SystemExit("No valid accuracy rows found")

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    write_svg(rows, out)
    print(out)


if __name__ == "__main__":
    main()
