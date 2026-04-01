#!/usr/bin/env python3
import argparse
import csv
from pathlib import Path


def load_rows(path: Path):
    rows = []
    with path.open("r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                np = int(row["np"])
                elapsed = float(row["elapsed_seconds"])
            except (ValueError, KeyError):
                continue
            rows.append((np, elapsed))
    rows.sort(key=lambda x: x[0])
    return rows


def write_svg(rows, out_path: Path):
    width, height = 900, 520
    margin_l, margin_r, margin_t, margin_b = 80, 30, 50, 70
    plot_w = width - margin_l - margin_r
    plot_h = height - margin_t - margin_b

    xs = [r[0] for r in rows]
    ys = [r[1] for r in rows]
    x_min, x_max = min(xs), max(xs)
    y_min, y_max = 0.0, max(ys) * 1.1
    if x_min == x_max:
        x_min -= 1
        x_max += 1

    def sx(x):
        return margin_l + (x - x_min) / (x_max - x_min) * plot_w

    def sy(y):
        return margin_t + (1 - (y - y_min) / (y_max - y_min)) * plot_h

    pts = " ".join(f"{sx(x):.1f},{sy(y):.1f}" for x, y in rows)

    y_ticks = 5
    tick_vals = [y_min + i * (y_max - y_min) / y_ticks for i in range(y_ticks + 1)]

    lines = []
    lines.append(f"<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>")
    lines.append("<rect width='100%' height='100%' fill='white' />")
    lines.append("<text x='450' y='28' text-anchor='middle' font-size='20' font-family='monospace'>MPI Scaling (Wall-Clock Sweep Time)</text>")

    for tv in tick_vals:
        y = sy(tv)
        lines.append(f"<line x1='{margin_l}' y1='{y:.1f}' x2='{width - margin_r}' y2='{y:.1f}' stroke='#e5e7eb' />")
        lines.append(f"<text x='{margin_l - 10}' y='{y + 4:.1f}' text-anchor='end' font-size='12' font-family='monospace' fill='#374151'>{tv:.2f}</text>")

    for x in xs:
        px = sx(x)
        lines.append(f"<line x1='{px:.1f}' y1='{margin_t}' x2='{px:.1f}' y2='{height - margin_b}' stroke='#f3f4f6' />")
        lines.append(f"<text x='{px:.1f}' y='{height - margin_b + 20}' text-anchor='middle' font-size='12' font-family='monospace' fill='#374151'>{x}</text>")

    lines.append(f"<line x1='{margin_l}' y1='{height - margin_b}' x2='{width - margin_r}' y2='{height - margin_b}' stroke='#111827' />")
    lines.append(f"<line x1='{margin_l}' y1='{margin_t}' x2='{margin_l}' y2='{height - margin_b}' stroke='#111827' />")
    lines.append(f"<polyline points='{pts}' fill='none' stroke='#2563eb' stroke-width='3' />")

    for x, y in rows:
        px, py = sx(x), sy(y)
        lines.append(f"<circle cx='{px:.1f}' cy='{py:.1f}' r='5' fill='#1d4ed8' />")
        lines.append(f"<text x='{px:.1f}' y='{py - 10:.1f}' text-anchor='middle' font-size='12' font-family='monospace' fill='#1f2937'>{y:.3f}s</text>")

    lines.append(f"<text x='{width/2:.1f}' y='{height - 18}' text-anchor='middle' font-size='14' font-family='monospace'>MPI ranks (np)</text>")
    lines.append(f"<text x='20' y='{height/2:.1f}' transform='rotate(-90 20 {height/2:.1f})' text-anchor='middle' font-size='14' font-family='monospace'>Elapsed seconds</text>")
    lines.append("</svg>")

    out_path.write_text("\n".join(lines), encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Plot MPI scaling from mpi_sweep_timings.csv")
    p.add_argument("timings_csv", help="Path to mpi_sweep_timings.csv")
    p.add_argument("--output", required=True, help="Output SVG path")
    args = p.parse_args()

    csv_path = Path(args.timings_csv)
    out_path = Path(args.output)
    rows = load_rows(csv_path)
    if not rows:
        raise SystemExit(f"No valid rows found in {csv_path}")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    write_svg(rows, out_path)
    print(out_path)


if __name__ == "__main__":
    main()
