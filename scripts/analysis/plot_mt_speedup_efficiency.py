#!/usr/bin/env python3
import argparse
import csv
from pathlib import Path


def load_rows(path: Path):
    serial = None
    mt = []
    with path.open("r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            mode = row.get("mode", "")
            try:
                threads = int(row["threads"])
                elapsed = float(row["elapsed_seconds"])
                sweep_raw = row.get("sweep_total_seconds", "")
                sweep = float(sweep_raw) if sweep_raw else None
            except (ValueError, KeyError):
                continue
            if mode == "serial":
                serial = (threads, elapsed, sweep)
            elif mode == "mt":
                mt.append((threads, elapsed, sweep))
    mt.sort(key=lambda x: x[0])
    return serial, mt


def write_svg(serial_elapsed, mt_rows, out_path: Path):
    width, height = 980, 560
    margin_l, margin_r, margin_t, margin_b = 90, 30, 60, 80
    plot_w = width - margin_l - margin_r
    plot_h = height - margin_t - margin_b

    xs = [t for t, _, _ in mt_rows]
    x_min, x_max = min(xs), max(xs)
    if x_min == x_max:
        x_min -= 1
        x_max += 1

    derived = []
    for t, e, _ in mt_rows:
        speedup = serial_elapsed / e if e > 0 else 0.0
        eff = speedup / t if t > 0 else 0.0
        derived.append((t, speedup, eff))

    y_max = max(max(v[1] for v in derived), 1.0) * 1.15

    def sx(x):
        return margin_l + (x - x_min) / (x_max - x_min) * plot_w

    def sy(y):
        return margin_t + (1 - y / y_max) * plot_h

    observed = " ".join(f"{sx(t):.1f},{sy(s):.1f}" for t, s, _ in derived)
    ideal = " ".join(f"{sx(t):.1f},{sy(float(t)):.1f}" for t, _, _ in derived)

    lines = []
    lines.append(f"<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>")
    lines.append("<rect width='100%' height='100%' fill='white' />")
    lines.append(
        f"<text x='{width/2:.1f}' y='30' text-anchor='middle' font-size='22' font-family='monospace'>MT Speedup and Efficiency</text>"
    )
    lines.append(
        f"<text x='{width/2:.1f}' y='52' text-anchor='middle' font-size='13' font-family='monospace' fill='#4b5563'>Baseline serial elapsed={serial_elapsed:.3f}s</text>"
    )

    for i in range(6):
        yv = y_max * i / 5
        y = sy(yv)
        lines.append(f"<line x1='{margin_l}' y1='{y:.1f}' x2='{width - margin_r}' y2='{y:.1f}' stroke='#e5e7eb' />")
        lines.append(f"<text x='{margin_l - 10}' y='{y + 4:.1f}' text-anchor='end' font-size='12' font-family='monospace' fill='#374151'>{yv:.2f}</text>")

    for t in xs:
        x = sx(t)
        lines.append(f"<line x1='{x:.1f}' y1='{margin_t}' x2='{x:.1f}' y2='{height - margin_b}' stroke='#f3f4f6' />")
        lines.append(f"<text x='{x:.1f}' y='{height - margin_b + 22}' text-anchor='middle' font-size='12' font-family='monospace' fill='#374151'>{t}</text>")

    lines.append(f"<line x1='{margin_l}' y1='{height - margin_b}' x2='{width - margin_r}' y2='{height - margin_b}' stroke='#111827' />")
    lines.append(f"<line x1='{margin_l}' y1='{margin_t}' x2='{margin_l}' y2='{height - margin_b}' stroke='#111827' />")

    lines.append(f"<polyline points='{ideal}' fill='none' stroke='#9ca3af' stroke-width='2' stroke-dasharray='6,6' />")
    lines.append(f"<polyline points='{observed}' fill='none' stroke='#2563eb' stroke-width='3' />")

    for t, s, e in derived:
        x, y = sx(t), sy(s)
        lines.append(f"<circle cx='{x:.1f}' cy='{y:.1f}' r='5' fill='#1d4ed8' />")
        lines.append(
            f"<text x='{x:.1f}' y='{y - 12:.1f}' text-anchor='middle' font-size='12' font-family='monospace' fill='#1f2937'>{s:.2f}x ({e*100:.1f}%)</text>"
        )

    lines.append(f"<text x='{width/2:.1f}' y='{height - 20}' text-anchor='middle' font-size='14' font-family='monospace'>Threads</text>")
    lines.append(f"<text x='22' y='{height/2:.1f}' transform='rotate(-90 22 {height/2:.1f})' text-anchor='middle' font-size='14' font-family='monospace'>Speedup (x)</text>")
    lines.append("</svg>")
    out_path.write_text("\n".join(lines), encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Plot MT speedup and efficiency")
    p.add_argument("timings_csv")
    p.add_argument("--output", required=True)
    args = p.parse_args()

    serial, mt = load_rows(Path(args.timings_csv))
    if serial is None or not mt:
        raise SystemExit("Need serial baseline row and at least one mt row")

    _, serial_elapsed, _ = serial
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    write_svg(serial_elapsed, mt, out)
    print(out)


if __name__ == "__main__":
    main()
