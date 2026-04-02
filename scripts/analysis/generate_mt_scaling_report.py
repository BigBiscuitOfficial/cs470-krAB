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
                mode = row["mode"]
                threads = int(row["threads"])
                elapsed = float(row["elapsed_seconds"])
                sweep_raw = row.get("sweep_total_seconds", "")
                sweep = float(sweep_raw) if sweep_raw else None
                strategy_sum_raw = row.get("strategy_total_sum_seconds", "")
                strategy_sum = float(strategy_sum_raw) if strategy_sum_raw else None
                strategy_pure_raw = row.get("strategy_pure_sum_seconds", "")
                strategy_pure_sum = float(strategy_pure_raw) if strategy_pure_raw else None
                run_dir = row.get("run_dir", "")
                profiling_csv = row.get("profiling_csv", "")
            except (ValueError, KeyError):
                continue
            rows.append(
                (
                    mode,
                    threads,
                    elapsed,
                    sweep,
                    strategy_sum,
                    strategy_pure_sum,
                    run_dir,
                    profiling_csv,
                )
            )
    return rows


def build_report(
    rows,
    timings_name: str,
    graph_name: str,
    time_log_graph_name: str | None,
    accuracy_csv_name: str | None,
    accuracy_graph_name: str | None,
):
    serial = [r for r in rows if r[0] == "serial"]
    mt = sorted([r for r in rows if r[0] == "mt"], key=lambda r: r[1])
    if not serial or not mt:
        raise ValueError("Need serial and mt rows")

    base = serial[0]
    (
        _,
        _,
        base_elapsed,
        base_sweep,
        base_strategy_sum,
        base_strategy_pure_sum,
        _,
        _,
    ) = base

    lines = []
    lines.append("# MT Scaling Report")
    lines.append("")
    lines.append("## Inputs")
    lines.append("")
    lines.append(f"- Timings CSV: `{timings_name}`")
    lines.append(f"- Serial baseline elapsed (wall-clock): {base_elapsed:.6f}s")
    if base_sweep is not None:
        lines.append(f"- Serial baseline sweep_total: {base_sweep:.6f}s")
    if base_strategy_sum is not None:
        lines.append(f"- Serial baseline strategy_total_sum: {base_strategy_sum:.6f}s")
    if base_strategy_pure_sum is not None:
        lines.append(
            f"- Serial baseline strategy_pure_sum: {base_strategy_pure_sum:.6f}s"
        )
    lines.append("")
    lines.append("## Results")
    lines.append("")
    lines.append("| mode | threads | elapsed_seconds | sweep_total_seconds | strategy_total_sum_seconds | strategy_pure_sum_seconds | speedup_vs_serial (wall) | speedup_vs_serial (pure_sum) | efficiency (wall) | efficiency (pure_sum) | run_dir |")
    lines.append("|:-----|--------:|----------------:|--------------------:|---------------------------:|--------------------------:|-------------------------:|------------------------------:|------------------:|-----------------------:|:--------|")

    base_sweep_display = f"{base_sweep:.6f}" if base_sweep is not None else ""
    base_strategy_display = (
        f"{base_strategy_sum:.6f}" if base_strategy_sum is not None else ""
    )
    base_strategy_pure_display = (
        f"{base_strategy_pure_sum:.6f}"
        if base_strategy_pure_sum is not None
        else ""
    )
    lines.append(
        f"| serial | 1 | {base_elapsed:.6f} | {base_sweep_display} | {base_strategy_display} | {base_strategy_pure_display} | 1.000x | 1.000x | 100.0% | 100.0% | `{base[6]}` |"
    )
    for (
        _,
        threads,
        elapsed,
        sweep,
        strategy_sum,
        strategy_pure_sum,
        run_dir,
        _,
    ) in mt:
        speedup_wall = base_elapsed / elapsed if elapsed > 0 else 0.0
        eff_wall = speedup_wall / threads if threads > 0 else 0.0

        if (
            base_strategy_sum is not None
            and strategy_sum is not None
            and strategy_sum > 0
        ):
            strategy_sum_text = f"{strategy_sum:.6f}"
        else:
            strategy_sum_text = ""

        if (
            base_strategy_pure_sum is not None
            and strategy_pure_sum is not None
            and strategy_pure_sum > 0
        ):
            speedup_pure = base_strategy_pure_sum / strategy_pure_sum
            eff_pure = speedup_pure / threads if threads > 0 else 0.0
            speedup_pure_text = f"{speedup_pure:.3f}x"
            eff_pure_text = f"{eff_pure*100:.1f}%"
            strategy_pure_text = f"{strategy_pure_sum:.6f}"
        else:
            speedup_pure_text = ""
            eff_pure_text = ""
            strategy_pure_text = ""

        sweep_text = f"{sweep:.6f}" if sweep is not None else ""

        lines.append(
            f"| mt | {threads} | {elapsed:.6f} | {sweep_text} | {strategy_sum_text} | {strategy_pure_text} | {speedup_wall:.3f}x | {speedup_pure_text} | {eff_wall*100:.1f}% | {eff_pure_text} | `{run_dir}` |"
        )

    lines.append("")
    lines.append("## Graph")
    lines.append("")
    lines.append(f"![MT speedup and efficiency]({graph_name})")
    lines.append("")

    if time_log_graph_name:
        lines.append("## Time Over Threads (log scale)")
        lines.append("")
        lines.append(f"![MT time over threads log scale]({time_log_graph_name})")
        lines.append("")

    if accuracy_csv_name:
        lines.append("## Accuracy Drift")
        lines.append("")
        lines.append(f"- Accuracy CSV: `{accuracy_csv_name}`")
        if accuracy_graph_name:
            lines.append("")
            lines.append(f"![MT accuracy drift]({accuracy_graph_name})")
        lines.append("")

    lines.append("## Notes")
    lines.append("")
    lines.append("- Efficiency = speedup / thread_count.")
    lines.append("- wall-clock includes process/setup overhead.")
    lines.append("- `sweep_total_seconds` can under-represent total work in MT strategy-level parallel runs due to overlapping strategy execution.")
    lines.append("- `strategy_total_sum_seconds` includes MT-overlapped overhead; useful for diagnosing coordination costs.")
    lines.append("- `strategy_pure_sum_seconds` (init + step_compute + metrics_calc) is the preferred work-comparable metric.")
    lines.append("- If wall-clock improves but pure_sum worsens, MT overhead/coordination is likely still too high.")
    lines.append("- Keep serial/mt seeds and strategy-space identical for fair comparisons.")
    return "\n".join(lines)


def main():
    p = argparse.ArgumentParser(description="Generate markdown MT scaling report")
    p.add_argument("timings_csv")
    p.add_argument("--graph", required=True)
    p.add_argument("--time-log-graph")
    p.add_argument("--accuracy-csv")
    p.add_argument("--accuracy-graph")
    p.add_argument("--output", required=True)
    args = p.parse_args()

    timings = Path(args.timings_csv)
    graph = Path(args.graph)
    time_log_graph = Path(args.time_log_graph) if args.time_log_graph else None
    accuracy_csv = Path(args.accuracy_csv) if args.accuracy_csv else None
    accuracy_graph = Path(args.accuracy_graph) if args.accuracy_graph else None
    out = Path(args.output)

    rows = load_rows(timings)
    report = build_report(
        rows,
        timings.name,
        graph.name,
        time_log_graph.name if time_log_graph else None,
        accuracy_csv.name if accuracy_csv else None,
        accuracy_graph.name if accuracy_graph else None,
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(report, encoding="utf-8")
    print(out)


if __name__ == "__main__":
    main()
