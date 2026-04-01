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
                run_dir = row.get("run_dir", "")
            except (ValueError, KeyError):
                continue
            rows.append((np, elapsed, run_dir))
    rows.sort(key=lambda x: x[0])
    return rows


def build_report(rows, timings_name: str, scaling_svg: str, speedup_svg: str):
    base_np, base_time, _ = rows[0]
    lines = []
    lines.append("# MPI Scaling Report")
    lines.append("")
    lines.append("## Inputs")
    lines.append("")
    lines.append(f"- Timings CSV: `{timings_name}`")
    lines.append(f"- Baseline: np={base_np}, elapsed={base_time:.6f}s")
    lines.append("")
    lines.append("## Results")
    lines.append("")
    lines.append("| np | elapsed_seconds | speedup_vs_baseline | efficiency | run_dir |")
    lines.append("|---:|----------------:|--------------------:|-----------:|:--------|")
    for np, elapsed, run_dir in rows:
        speedup = base_time / elapsed if elapsed > 0 else 0.0
        efficiency = speedup / (np / base_np) if np > 0 else 0.0
        lines.append(
            f"| {np} | {elapsed:.6f} | {speedup:.3f}x | {efficiency*100:.1f}% | `{run_dir}` |"
        )

    lines.append("")
    lines.append("## Graphs")
    lines.append("")
    lines.append("### Wall-Clock Scaling")
    lines.append("")
    lines.append(f"![MPI wall-clock scaling]({scaling_svg})")
    lines.append("")
    lines.append("### Speedup and Efficiency")
    lines.append("")
    lines.append(f"![MPI speedup and efficiency]({speedup_svg})")
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append("- Efficiency is computed relative to the lowest-NP baseline in this file.")
    lines.append("- Use larger workloads for MPI scaling claims to avoid startup/communication overhead distortion.")
    return "\n".join(lines)


def main():
    p = argparse.ArgumentParser(description="Generate markdown MPI scaling report")
    p.add_argument("timings_csv", help="Path to mpi_sweep_timings.csv")
    p.add_argument("--scaling-svg", required=True, help="Path to mpi_scaling.svg")
    p.add_argument(
        "--speedup-svg", required=True, help="Path to mpi_speedup_efficiency.svg"
    )
    p.add_argument("--output", required=True, help="Output markdown path")
    args = p.parse_args()

    timings = Path(args.timings_csv)
    scaling_svg = Path(args.scaling_svg)
    speedup_svg = Path(args.speedup_svg)
    output = Path(args.output)

    rows = load_rows(timings)
    if not rows:
        raise SystemExit(f"No valid rows found in {timings}")

    output.parent.mkdir(parents=True, exist_ok=True)
    report = build_report(
        rows,
        timings.name,
        scaling_svg.name,
        speedup_svg.name,
    )
    output.write_text(report, encoding="utf-8")
    print(output)


if __name__ == "__main__":
    main()
