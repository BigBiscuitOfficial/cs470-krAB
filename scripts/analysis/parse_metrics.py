#!/usr/bin/env python3
"""
Performance Metrics Parser for KrAB Profiling Data

Reads one or more profiling CSV files, validates schema compliance,
calculates derived metrics (speedup, efficiency, load imbalance),
and emits a consolidated CSV for analysis and plotting.

Usage:
    python parse_metrics.py <input_csv>... [options]

Examples:
    # Parse single file
    python parse_metrics.py profiling_serial_001.csv

    # Parse multiple files and consolidate
    python parse_metrics.py output/profiling_*.csv --output consolidated.csv

    # Calculate speedup relative to serial baseline
    python parse_metrics.py profiling_*.csv --baseline serial --output scaling.csv

    # Validate only (no output)
    python parse_metrics.py profiling_*.csv --validate-only
"""

import argparse
import csv
import sys
from pathlib import Path
from typing import List, Dict, Optional
from collections import defaultdict


# Required columns per METRICS_SCHEMA.md
REQUIRED_COLUMNS = [
    "mode",
    "num_agents",
    "num_steps",
    "total_cores",
    "total_runtime_s",
]

# All expected columns
ALL_COLUMNS = REQUIRED_COLUMNS + [
    "run_id",
    "num_reps",
    "num_threads",
    "num_ranks",
    "init_time_s",
    "step_compute_s",
    "comm_overhead_s",
    "metrics_calc_s",
    "strategy_desc",
    "hostname",
    "timestamp",
]

# Optional derived metric columns
DERIVED_COLUMNS = [
    "agents_per_step_per_s",
    "speedup",
    "efficiency",
    "load_imbalance",
]


class ValidationError(Exception):
    """Raised when CSV validation fails."""
    pass


def _as_int(value: Optional[str], default: int = 0) -> int:
    """Parse int with a default for empty/missing values."""
    if value is None:
        return default
    text = str(value).strip()
    if not text:
        return default
    return int(text)


def _as_float(value: Optional[str], default: float = 0.0) -> float:
    """Parse float with a default for empty/missing values."""
    if value is None:
        return default
    text = str(value).strip()
    if not text:
        return default
    return float(text)


def normalize_row(row: Dict[str, str]) -> Dict[str, str]:
    """
    Normalize row fields so parsing tolerates partial/current profiling formats.

    Keeps required validation strict while backfilling optional fields when missing.
    """
    normalized = dict(row)

    num_threads = _as_int(normalized.get("num_threads"), default=0)
    num_ranks = _as_int(normalized.get("num_ranks"), default=0)
    total_cores = _as_int(normalized.get("total_cores"), default=0)

    if total_cores <= 0:
        if num_threads > 0 and num_ranks > 0:
            total_cores = num_threads * num_ranks
        elif num_threads > 0:
            total_cores = num_threads
        elif num_ranks > 0:
            total_cores = num_ranks
        else:
            total_cores = 1

    if num_ranks <= 0:
        num_ranks = 1
    if num_threads <= 0:
        num_threads = max(1, total_cores // num_ranks)

    normalized.setdefault("num_reps", "1")
    normalized["num_threads"] = str(num_threads)
    normalized["num_ranks"] = str(num_ranks)
    normalized["total_cores"] = str(total_cores)

    for col in ["init_time_s", "step_compute_s", "comm_overhead_s", "metrics_calc_s"]:
        if not str(normalized.get(col, "")).strip():
            normalized[col] = "0"

    normalized.setdefault("strategy_desc", "")
    normalized.setdefault("hostname", "unknown")
    normalized.setdefault("timestamp", "")
    normalized.setdefault("run_id", "")

    return normalized


def is_aggregate_runtime_row(row: Dict[str, str]) -> bool:
    """Return True for rows that represent a full runtime measurement."""
    event = row.get("event", "").strip().lower()
    if not event:
        return True
    return event in {"strategy_total", "sweep_total", "run_duration"}


def validate_row(row: Dict[str, str], row_num: int, filename: str) -> List[str]:
    """
    Validate a single row against schema requirements.
    Returns list of warning messages (empty if valid).
    """
    warnings_list = []

    # Check required columns present and non-empty
    for col in REQUIRED_COLUMNS:
        if col not in row or not row[col].strip():
            raise ValidationError(
                f"{filename}:{row_num}: Missing required column '{col}'"
            )

    # Type and range checks
    try:
        total_runtime = _as_float(row.get("total_runtime_s"), 0.0)
        init_time = _as_float(row.get("init_time_s"), 0.0)
        step_compute = _as_float(row.get("step_compute_s"), 0.0)
        metrics_time = _as_float(row.get("metrics_calc_s"), 0.0)
        comm_overhead = _as_float(row.get("comm_overhead_s"), 0.0)

        # Sanity check: total >= sum of components
        sum_components = init_time + step_compute + metrics_time + comm_overhead
        if total_runtime < sum_components - 0.001:  # Small epsilon for float precision
            warnings_list.append(
                f"Row {row_num}: total_runtime_s ({total_runtime:.3f}s) < "
                f"sum of components ({sum_components:.3f}s)"
            )

        # Check for negative values
        if comm_overhead < 0:
            warnings_list.append(
                f"Row {row_num}: comm_overhead_s is negative ({comm_overhead:.3f}s)"
            )

        # Check core count consistency
        if "num_threads" in row and "num_ranks" in row and "total_cores" in row:
            num_threads = _as_int(row.get("num_threads"), 1)
            num_ranks = _as_int(row.get("num_ranks"), 1)
            total_cores = _as_int(row.get("total_cores"), 1)
            expected_cores = num_threads * num_ranks
            if total_cores != expected_cores:
                warnings_list.append(
                    f"Row {row_num}: total_cores ({total_cores}) != "
                    f"num_threads ({num_threads}) * num_ranks ({num_ranks})"
                )

    except ValueError as e:
        raise ValidationError(f"{filename}:{row_num}: Invalid numeric value: {e}")

    return warnings_list


def calculate_derived_metrics(
    row: Dict[str, str],
    baseline_time: Optional[float] = None
) -> Dict[str, float]:
    """
    Calculate derived performance metrics.

    Args:
        row: CSV row as dictionary
        baseline_time: Baseline runtime for speedup calculation (optional)

    Returns:
        Dictionary with derived metric values
    """
    derived = {}

    num_agents = _as_int(row.get("num_agents"), 0)
    num_steps = _as_int(row.get("num_steps"), 0)
    total_runtime = _as_float(row.get("total_runtime_s"), 0.0)
    total_cores = _as_int(row.get("total_cores"), 1)

    if total_runtime <= 0:
        derived["agents_per_step_per_s"] = None
        derived["speedup"] = None
        derived["efficiency"] = None
        derived["overhead_fraction"] = None
        derived["load_imbalance"] = None
        return derived

    if not is_aggregate_runtime_row(row):
        derived["agents_per_step_per_s"] = None
        derived["speedup"] = None
        derived["efficiency"] = None
        derived["overhead_fraction"] = None
        derived["load_imbalance"] = None
        return derived

    # Throughput: agent-steps per second
    derived["agents_per_step_per_s"] = (num_agents * num_steps) / total_runtime

    # Speedup and efficiency (if baseline provided)
    if baseline_time is not None and baseline_time > 0:
        speedup = baseline_time / total_runtime
        efficiency = speedup / total_cores
        derived["speedup"] = speedup
        derived["efficiency"] = efficiency
    else:
        derived["speedup"] = 1.0 if total_cores == 1 else None
        derived["efficiency"] = 1.0 if total_cores == 1 else None

    # Overhead fraction
    if "comm_overhead_s" in row and row["comm_overhead_s"]:
        comm_overhead = float(row["comm_overhead_s"])
        derived["overhead_fraction"] = comm_overhead / total_runtime
    else:
        derived["overhead_fraction"] = None

    # Load imbalance (if per-thread data available)
    # Note: Current schema doesn't include per-thread times in CSV
    # This would require reading from JSON profiling data
    derived["load_imbalance"] = None

    return derived


def find_baseline(records: List[Dict[str, str]], mode: str = "serial") -> Optional[Dict]:
    """
    Find baseline record for speedup calculation.

    Args:
        records: List of all parsed records
        mode: Execution mode to use as baseline (default: "serial")

    Returns:
        Baseline record dictionary or None if not found
    """
    # Find all records matching baseline mode
    baseline_candidates = [
        r for r in records
        if r["mode"] == mode and is_aggregate_runtime_row(r)
    ]

    if not baseline_candidates:
        return None

    # If multiple baselines, group by problem size and take first of each
    by_problem = defaultdict(list)
    for record in baseline_candidates:
        key = (int(record["num_agents"]), int(record["num_steps"]))
        by_problem[key].append(record)

    # Return dict of baseline times keyed by problem size
    baselines = {}
    for problem_size, records_list in by_problem.items():
        # Use minimum runtime if multiple runs (optimistic baseline)
        best = min(records_list, key=lambda r: float(r["total_runtime_s"]))
        baselines[problem_size] = float(best["total_runtime_s"])

    return baselines


def parse_csv_file(filepath: Path, validate: bool = True) -> List[Dict[str, str]]:
    """
    Parse a single CSV file and optionally validate.

    Args:
        filepath: Path to CSV file
        validate: Whether to validate schema compliance

    Returns:
        List of row dictionaries

    Raises:
        ValidationError: If validation fails
    """
    records = []
    all_warnings = []

    try:
        with open(filepath, "r", encoding="utf-8") as f:
            reader = csv.DictReader(f)

            # Check for required columns in header
            if reader.fieldnames is None:
                raise ValidationError(f"{filepath}: Empty or invalid CSV file")

            missing_required = set(REQUIRED_COLUMNS) - set(reader.fieldnames)
            if missing_required:
                raise ValidationError(
                    f"{filepath}: Missing required columns: {missing_required}"
                )

            # Parse rows
            for row_num, row in enumerate(reader, start=2):  # Line 2 = first data row
                normalized_row = normalize_row(row)

                if validate:
                    warnings_list = validate_row(normalized_row, row_num, str(filepath))
                    all_warnings.extend(warnings_list)

                records.append(normalized_row)

    except FileNotFoundError:
        raise ValidationError(f"File not found: {filepath}")
    except csv.Error as e:
        raise ValidationError(f"{filepath}: CSV parsing error: {e}")

    # Print accumulated warnings
    if all_warnings:
        print(f"⚠️  Warnings for {filepath}:", file=sys.stderr)
        for warning in all_warnings:
            print(f"   {warning}", file=sys.stderr)

    return records


def write_consolidated_csv(
    records: List[Dict],
    output_path: Path,
    include_derived: bool = True
):
    """
    Write consolidated CSV with all records and derived metrics.

    Args:
        records: List of record dictionaries (with derived metrics added)
        output_path: Output CSV path
        include_derived: Whether to include derived metric columns
    """
    if not records:
        print("⚠️  No records to write", file=sys.stderr)
        return

    # Determine output columns (preserve input order, add derived at end)
    sample_record = records[0]
    input_cols = [k for k in sample_record.keys() if k not in DERIVED_COLUMNS]
    derived_cols = [k for k in DERIVED_COLUMNS if k in sample_record]
    output_cols = input_cols + (derived_cols if include_derived else [])

    with open(output_path, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=output_cols, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(records)

    print(f"✓ Wrote {len(records)} records to {output_path}")


def print_summary_stats(records: List[Dict]):
    """Print summary statistics of parsed data."""
    print("\n" + "=" * 60)
    print("PROFILING DATA SUMMARY")
    print("=" * 60)

    # Group by mode
    by_mode = defaultdict(list)
    for record in records:
        by_mode[record["mode"]].append(record)

    print(f"\nTotal records: {len(records)}")
    print(f"Execution modes: {list(by_mode.keys())}")

    for mode, mode_records in sorted(by_mode.items()):
        print(f"\n{mode.upper()}:")
        print(f"  Count: {len(mode_records)}")

        runtimes = [float(r["total_runtime_s"]) for r in mode_records]
        cores = [int(r["total_cores"]) for r in mode_records]

        print(f"  Runtime range: {min(runtimes):.3f}s - {max(runtimes):.3f}s")
        print(f"  Core counts: {sorted(set(cores))}")

        # If efficiency calculated, show range
        efficiencies = [
            r["efficiency"] for r in mode_records
            if r.get("efficiency") is not None
        ]
        if efficiencies:
            print(f"  Efficiency range: {min(efficiencies):.3f} - {max(efficiencies):.3f}")

    print("\n" + "=" * 60 + "\n")


def main():
    parser = argparse.ArgumentParser(
        description="Parse and validate KrAB profiling metrics CSV files",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "input_files",
        nargs="+",
        type=Path,
        help="Input CSV file(s) to parse (glob patterns supported)",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Output consolidated CSV file (default: print to stdout)",
    )
    parser.add_argument(
        "--baseline",
        type=str,
        default="serial",
        help="Execution mode to use as baseline for speedup (default: serial)",
    )
    parser.add_argument(
        "--no-validate",
        action="store_true",
        help="Skip validation checks",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Only validate, do not produce output",
    )
    parser.add_argument(
        "--no-derived",
        action="store_true",
        help="Do not calculate derived metrics",
    )
    parser.add_argument(
        "-q",
        "--quiet",
        action="store_true",
        help="Suppress summary statistics",
    )

    args = parser.parse_args()

    # Parse all input files
    all_records = []
    validation_failed = False

    print(f"Parsing {len(args.input_files)} file(s)...")

    for filepath in args.input_files:
        if not filepath.exists():
            print(f"✗ File not found: {filepath}", file=sys.stderr)
            validation_failed = True
            continue

        try:
            records = parse_csv_file(filepath, validate=not args.no_validate)
            all_records.extend(records)
            print(f"✓ Parsed {filepath}: {len(records)} records")
        except ValidationError as e:
            print(f"✗ {e}", file=sys.stderr)
            validation_failed = True
        except Exception as e:
            print(f"✗ Unexpected error parsing {filepath}: {e}", file=sys.stderr)
            validation_failed = True

    if validation_failed:
        sys.exit(1)

    if not all_records:
        print("✗ No valid records found", file=sys.stderr)
        sys.exit(1)

    if args.validate_only:
        print(f"\n✓ Validation passed for {len(all_records)} total records")
        sys.exit(0)

    # Calculate derived metrics
    if not args.no_derived:
        baselines = find_baseline(all_records, mode=args.baseline)

        if baselines:
            print(f"\n✓ Found {len(baselines)} baseline(s) for mode '{args.baseline}'")
        else:
            print(
                f"\n⚠️  No baseline records found for mode '{args.baseline}' "
                f"(speedup/efficiency will be None)",
                file=sys.stderr,
            )

        for record in all_records:
            problem_size = (int(record["num_agents"]), int(record["num_steps"]))
            baseline_time = baselines.get(problem_size) if baselines else None

            derived = calculate_derived_metrics(record, baseline_time)
            record.update(derived)

    # Print summary
    if not args.quiet:
        print_summary_stats(all_records)

    # Write output
    if args.output:
        write_consolidated_csv(
            all_records,
            args.output,
            include_derived=not args.no_derived,
        )
    else:
        # Print to stdout
        if all_records:
            writer = csv.DictWriter(sys.stdout, fieldnames=all_records[0].keys())
            writer.writeheader()
            writer.writerows(all_records)


if __name__ == "__main__":
    main()
