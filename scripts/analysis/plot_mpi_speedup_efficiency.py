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
    base_np, base_t = rows[0]
    derived = []
    for np, t in rows:
        speedup = base_t / t if t > 0 else 0.0
        efficiency = (speedup / (np / base_np)) if np > 0 else 0.0
        derived.append((np, speedup, efficiency))

    width, height = 980, 560
    margin_l, margin_r, margin_t, margin_b = 90, 30, 60, 80
    plot_w = width - margin_l - margin_r
    plot_h = height - margin_t - margin_b

    xs = [r[0] for r in derived]
    x_min, x_max = min(xs), max(xs)
    if x_min == x_max:
        x_min -= 1
        x_max += 1

    y_max = max(max(r[1] for r in derived), 1.0) * 1.15

    def sx(x):
        return margin_l + (x - x_min) / (x_max - x_min) * plot_w

    def sy(y):
        return margin_t + (1 - y / y_max) * plot_h

    speed_pts = " ".join(f"{sx(np):.1f},{sy(sp):.1f}" for np, sp, _ in derived)
    ideal_pts = " ".join(f"{sx(np):.1f},{sy(np / base_np):.1f}" for np, _, _ in derived)

    lines = []
    lines.append(f"<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>")
    lines.append("<rect width='100%' height='100%' fill='white' />")
    lines.append(
        f"<text x='{width/2:.1f}' y='30' text-anchor='middle' font-size='22' font-family='monospace'>MPI Speedup and Efficiency</text>"
    )
    lines.append(
        f"<text x='{width/2:.1f}' y='52' text-anchor='middle' font-size='13' font-family='monospace' fill='#4b5563'>Baseline: np={base_np}, elapsed={base_t:.3f}s</text>"
    )

    for i in range(6):
        yv = y_max * i / 5
        y = sy(yv)
        lines.append(f"<line x1='{margin_l}' y1='{y:.1f}' x2='{width - margin_r}' y2='{y:.1f}' stroke='#e5e7eb' />")
        lines.append(
            f"<text x='{margin_l - 10}' y='{y + 4:.1f}' text-anchor='end' font-size='12' font-family='monospace' fill='#374151'>{yv:.2f}</text>"
        )

    for np in xs:
        x = sx(np)
        lines.append(f"<line x1='{x:.1f}' y1='{margin_t}' x2='{x:.1f}' y2='{height - margin_b}' stroke='#f3f4f6' />")
        lines.append(
            f"<text x='{x:.1f}' y='{height - margin_b + 22}' text-anchor='middle' font-size='12' font-family='monospace' fill='#374151'>{np}</text>"
        )

    lines.append(f"<line x1='{margin_l}' y1='{height - margin_b}' x2='{width - margin_r}' y2='{height - margin_b}' stroke='#111827' />")
    lines.append(f"<line x1='{margin_l}' y1='{margin_t}' x2='{margin_l}' y2='{height - margin_b}' stroke='#111827' />")

    lines.append(f"<polyline points='{ideal_pts}' fill='none' stroke='#9ca3af' stroke-width='2' stroke-dasharray='6,6' />")
    lines.append(f"<polyline points='{speed_pts}' fill='none' stroke='#2563eb' stroke-width='3' />")

    for np, sp, eff in derived:
        x, y = sx(np), sy(sp)
        lines.append(f"<circle cx='{x:.1f}' cy='{y:.1f}' r='5' fill='#1d4ed8' />")
        lines.append(
            f"<text x='{x:.1f}' y='{y - 12:.1f}' text-anchor='middle' font-size='12' font-family='monospace' fill='#1f2937'>{sp:.2f}x ({eff*100:.1f}%)</text>"
        )

    lines.append(f"<text x='{width/2:.1f}' y='{height - 20}' text-anchor='middle' font-size='14' font-family='monospace'>MPI ranks (np)</text>")
    lines.append(
        f"<text x='22' y='{height/2:.1f}' transform='rotate(-90 22 {height/2:.1f})' text-anchor='middle' font-size='14' font-family='monospace'>Speedup (x)</text>"
    )

    legend_y = margin_t + 10
    legend_x = width - margin_r - 250
    lines.append(f"<line x1='{legend_x}' y1='{legend_y}' x2='{legend_x + 36}' y2='{legend_y}' stroke='#2563eb' stroke-width='3' />")
    lines.append(
        f"<text x='{legend_x + 44}' y='{legend_y + 4}' font-size='12' font-family='monospace' fill='#1f2937'>Observed speedup</text>"
    )
    lines.append(
        f"<line x1='{legend_x}' y1='{legend_y + 18}' x2='{legend_x + 36}' y2='{legend_y + 18}' stroke='#9ca3af' stroke-width='2' stroke-dasharray='6,6' />"
    )
    lines.append(
        f"<text x='{legend_x + 44}' y='{legend_y + 22}' font-size='12' font-family='monospace' fill='#1f2937'>Ideal linear speedup</text>"
    )

    lines.append("</svg>")
    out_path.write_text("\n".join(lines), encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Plot MPI speedup and efficiency")
    p.add_argument("timings_csv", help="Path to mpi_sweep_timings.csv")
    p.add_argument("--output", required=True, help="Output SVG path")
    args = p.parse_args()

    rows = load_rows(Path(args.timings_csv))
    if len(rows) < 2:
        raise SystemExit("Need at least two NP points to compute speedup/efficiency")

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    write_svg(rows, out_path)
    print(out_path)


if __name__ == "__main__":
    main()
