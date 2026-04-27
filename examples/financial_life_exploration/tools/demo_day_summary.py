#!/usr/bin/env python3
"""Helpers for the demo-day MPI scaling script."""

import argparse
import csv
import re
from pathlib import Path
from typing import Dict, List, Optional


def as_float(row: Dict[str, str], key: str, default: float = 0.0) -> float:
    try:
        return float(row.get(key, default))
    except (TypeError, ValueError):
        return default


def parse_log_metrics(log_path: Path) -> Dict[str, str]:
    total_max: List[float] = []
    overheads: List[float] = []
    total_run = "N/A"
    best_fitness = "N/A"

    with log_path.open("r", encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("[MPI_TIMING]"):
                total_match = re.search(r"total_max=([0-9.]+)s", line)
                overhead_match = re.search(r"overhead_ratio=([0-9.]+)", line)
                if total_match:
                    total_max.append(float(total_match.group(1)))
                if overhead_match:
                    overheads.append(float(overhead_match.group(1)))
            elif "Completed generation" in line:
                run_match = re.search(r"after ([0-9.]+) seconds", line)
                if run_match:
                    total_run = run_match.group(1)
            elif "- Overall best fitness is" in line:
                fitness_match = re.search(r"is ([0-9.]+)", line)
                if fitness_match:
                    best_fitness = fitness_match.group(1)

    avg_total = sum(total_max) / len(total_max) if total_max else 0.0
    avg_overhead = sum(overheads) / len(overheads) if overheads else 0.0
    return {
        "mpi_avg_total_seconds": f"{avg_total:.6f}",
        "avg_overhead_ratio": f"{avg_overhead:.6f}",
        "total_run_seconds": total_run,
        "best_fitness": best_fitness,
    }


def print_log_metrics(args: argparse.Namespace) -> None:
    metrics = parse_log_metrics(Path(args.log_file))
    print(
        "\t".join(
            [
                metrics["mpi_avg_total_seconds"],
                metrics["avg_overhead_ratio"],
                metrics["total_run_seconds"],
                metrics["best_fitness"],
            ]
        )
    )


def calculate_speedup(args: argparse.Namespace) -> None:
    baseline = float(args.baseline_mpi_avg_total)
    current = float(args.current_mpi_avg_total)
    print(f"{baseline / current:.3f}" if current > 0 else "N/A")


def calculate_efficiency(args: argparse.Namespace) -> None:
    try:
        speedup = float(args.speedup)
        ranks = float(args.ranks)
        baseline_ranks = float(args.baseline_ranks)
    except ValueError:
        print("N/A")
        return

    scale_factor = ranks / baseline_ranks
    print(f"{speedup / scale_factor:.3f}" if scale_factor > 0 else "N/A")


def load_rows(summary_path: Path) -> List[Dict[str, str]]:
    with summary_path.open("r", encoding="utf-8") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def extract_strategy_lines(interpretation_path: Path) -> List[str]:
    interpretation = interpretation_path.read_text(encoding="utf-8").strip().splitlines()
    interesting_prefixes = (
        "Best fitness:",
        "Scale config:",
        "Winning genome:",
        "Policy profile:",
    )
    strategy_lines = [
        line for line in interpretation if line.startswith(interesting_prefixes)
    ]

    why_lines: List[str] = []
    in_why = False
    for line in interpretation:
        if line == "Why this policy likely won:":
            in_why = True
            continue
        if in_why:
            if not line.strip() or line.startswith("Demo takeaway:"):
                break
            why_lines.append(line)

    if why_lines:
        strategy_lines.append("Why it likely won:")
        strategy_lines.extend(f"  {line}" for line in why_lines)

    return strategy_lines


def best_agent_interpretation(
    rows: List[Dict[str, str]], fallback_path: Path
) -> tuple[Optional[Dict[str, str]], Path]:
    fitness_rows = [
        row for row in rows if row.get("best_fitness") not in {"", "N/A", None}
    ]
    best_agent = (
        min(fitness_rows, key=lambda row: as_float(row, "best_fitness", float("inf")))
        if fitness_rows
        else None
    )

    if best_agent:
        interpretation_path = (
            Path("outcomes")
            / f"financial_interpretation_{best_agent['procs']}_procs.txt"
        )
        if interpretation_path.exists():
            return best_agent, interpretation_path

    return best_agent, fallback_path


def build_demo_summary(args: argparse.Namespace) -> None:
    summary_path = Path(args.summary_file)
    fallback_interpretation_path = Path(args.baseline_interpretation)
    output_path = Path(args.output)
    rows = load_rows(summary_path)

    best_speedup = max(rows, key=lambda row: as_float(row, "speedup")) if rows else None
    best_efficiency = (
        max(rows, key=lambda row: as_float(row, "efficiency")) if rows else None
    )
    fastest_wall = (
        min(rows, key=lambda row: as_float(row, "wall_elapsed_seconds", float("inf")))
        if rows
        else None
    )
    best_agent, best_interpretation_path = best_agent_interpretation(
        rows, fallback_interpretation_path
    )
    baseline = rows[0] if rows else None
    last = rows[-1] if rows else None
    matching = sum(
        1
        for row in rows
        if row.get("interpretation_status") in {"baseline", "matches_baseline"}
    )

    lines = [
        "Demo-Day Summary",
        f"This was the strategy taken by the best agent on seed {args.seed}.",
    ]
    if best_agent:
        lines.append(
            "Best agent selected from the scaling sweep: "
            f"{best_agent['procs']} process(es), fitness {best_agent['best_fitness']}."
        )

    lines.extend(["", "Best-agent strategy:"])
    lines.extend(f"  {line}" for line in extract_strategy_lines(best_interpretation_path))

    lines.extend(["", "Parallel and distributed systems takeaways:"])
    if baseline:
        lines.append(
            "  Baseline: "
            f"{baseline['procs']} process, "
            f"MPI avg generation time {baseline['mpi_avg_total_seconds']}s, "
            f"wall time {baseline['wall_elapsed_seconds']}s."
        )
    if best_speedup:
        lines.append(
            "  Best speedup: "
            f"{best_speedup['speedup']}x at {best_speedup['procs']} processes "
            f"with efficiency {best_speedup['efficiency']}."
        )
    if best_efficiency:
        lines.append(
            "  Best parallel efficiency: "
            f"{best_efficiency['efficiency']} at {best_efficiency['procs']} processes."
        )
    if fastest_wall:
        lines.append(
            "  Fastest wall-clock run: "
            f"{fastest_wall['wall_elapsed_seconds']}s at "
            f"{fastest_wall['procs']} processes."
        )
    if last:
        lines.append(
            "  Largest run shown: "
            f"{last['procs']} processes, speedup {last['speedup']}x, "
            f"efficiency {last['efficiency']}, overhead ratio "
            f"{last['avg_overhead_ratio']}."
        )
    if rows:
        lines.append(
            f"  Determinism check: {matching}/{len(rows)} interpretations matched "
            "the baseline best-agent explanation."
        )

    lines.extend(
        [
            "",
            "Scaling table:",
            "  procs | mpi_avg_s | speedup | efficiency | overhead | wall_s | "
            "best_fitness | status",
        ]
    )
    for row in rows:
        lines.append(
            "  {procs:>5} | {mpi_avg_total_seconds:>9} | {speedup:>7} | "
            "{efficiency:>10} | {avg_overhead_ratio:>8} | "
            "{wall_elapsed_seconds:>6} | {best_fitness:>12} | "
            "{interpretation_status}".format(**row)
        )

    output_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    metrics = subparsers.add_parser("metrics", help="Parse one run log.")
    metrics.add_argument("log_file")
    metrics.set_defaults(func=print_log_metrics)

    speedup = subparsers.add_parser("speedup", help="Calculate strong-scaling speedup.")
    speedup.add_argument("baseline_mpi_avg_total")
    speedup.add_argument("current_mpi_avg_total")
    speedup.set_defaults(func=calculate_speedup)

    efficiency = subparsers.add_parser(
        "efficiency", help="Calculate strong-scaling efficiency."
    )
    efficiency.add_argument("speedup")
    efficiency.add_argument("ranks")
    efficiency.add_argument("baseline_ranks")
    efficiency.set_defaults(func=calculate_efficiency)

    summary = subparsers.add_parser("summary", help="Build the demo-day summary.")
    summary.add_argument("summary_file")
    summary.add_argument("baseline_interpretation")
    summary.add_argument("output")
    summary.add_argument("seed")
    summary.set_defaults(func=build_demo_summary)

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    args.func(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
