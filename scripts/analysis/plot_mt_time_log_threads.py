#!/usr/bin/env python3
import argparse
import csv
import math
from pathlib import Path


def load_rows(path: Path):
    serial = None
    mt = []
    with path.open("r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                mode = row["mode"]
                threads = int(row["threads"])
                elapsed = float(row["elapsed_seconds"])
            except (ValueError, KeyError):
                continue
            if mode == "serial":
                serial = (threads, elapsed)
            elif mode == "mt":
                mt.append((threads, elapsed))
    mt.sort(key=lambda x: x[0])
    return serial, mt


def write_svg(serial, mt, out: Path):
    width, height = 920, 540
    ml, mr, mtop, mb = 90, 40, 60, 80
    pw, ph = width - ml - mr, height - mtop - mb

    points = []
    if serial is not None:
        points.append((serial[0], serial[1], "serial"))
    points.extend((t, e, "mt") for t, e in mt)

    xs = [p[0] for p in points]
    ys = [p[1] for p in points if p[1] > 0]
    x_min, x_max = min(xs), max(xs)
    if x_min == x_max:
        x_min -= 1
        x_max += 1

    y_min = math.log10(min(ys) * 0.9)
    y_max = math.log10(max(ys) * 1.1)

    def sx(x):
        return ml + (x - x_min) / (x_max - x_min) * pw

    def sy(y):
        ly = math.log10(y)
        return mtop + (1 - (ly - y_min) / (y_max - y_min)) * ph

    lines = []
    lines.append(f"<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>")
    lines.append("<rect width='100%' height='100%' fill='white' />")
    lines.append("<text x='460' y='30' text-anchor='middle' font-size='22' font-family='monospace'>Time vs Threads (log scale)</text>")

    for i in range(6):
        lv = y_min + (y_max - y_min) * i / 5
        v = 10 ** lv
        y = mtop + (1 - i / 5) * ph
        lines.append(f"<line x1='{ml}' y1='{y:.1f}' x2='{width-mr}' y2='{y:.1f}' stroke='#e5e7eb' />")
        lines.append(f"<text x='{ml-10}' y='{y+4:.1f}' text-anchor='end' font-size='12' font-family='monospace'>{v:.3f}s</text>")

    for x in sorted(set(xs)):
        xx = sx(x)
        lines.append(f"<line x1='{xx:.1f}' y1='{mtop}' x2='{xx:.1f}' y2='{height-mb}' stroke='#f3f4f6' />")
        lines.append(f"<text x='{xx:.1f}' y='{height-mb+22}' text-anchor='middle' font-size='12' font-family='monospace'>{x}</text>")

    lines.append(f"<line x1='{ml}' y1='{height-mb}' x2='{width-mr}' y2='{height-mb}' stroke='#111827' />")
    lines.append(f"<line x1='{ml}' y1='{mtop}' x2='{ml}' y2='{height-mb}' stroke='#111827' />")

    mt_points = [(t, e) for t, e, tag in points if tag == "mt"]
    if mt_points:
        poly = " ".join(f"{sx(t):.1f},{sy(e):.1f}" for t, e in mt_points)
        lines.append(f"<polyline points='{poly}' fill='none' stroke='#2563eb' stroke-width='3' />")

    for t, e, tag in points:
        x, y = sx(t), sy(e)
        color = "#dc2626" if tag == "serial" else "#1d4ed8"
        label = "serial" if tag == "serial" else f"{t}t"
        lines.append(f"<circle cx='{x:.1f}' cy='{y:.1f}' r='5' fill='{color}' />")
        lines.append(f"<text x='{x:.1f}' y='{y-10:.1f}' text-anchor='middle' font-size='12' font-family='monospace'>{label}:{e:.3f}s</text>")

    lines.append("</svg>")
    out.write_text("\n".join(lines), encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Plot time over threads on logarithmic scale")
    p.add_argument("timings_csv")
    p.add_argument("--output", required=True)
    args = p.parse_args()

    serial, mt = load_rows(Path(args.timings_csv))
    if serial is None and not mt:
        raise SystemExit("No timing rows found")

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    write_svg(serial, mt, out)
    print(out)


if __name__ == "__main__":
    main()
