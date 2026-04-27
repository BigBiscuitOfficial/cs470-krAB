#!/usr/bin/env python3
"""Interpret a financial_life_exploration run log in plain English."""

import argparse
import re
from pathlib import Path
from typing import Dict, List, NamedTuple


GENE_NAMES = [
    "frugality",
    "savings_discipline",
    "career_drive",
    "risk_tolerance",
    "resilience",
    "family_pressure",
    "education_investment",
]


class RunSummary(NamedTuple):
    config: Dict[str, str]
    best_generation: int
    best_index: int
    best_fitness: float
    genome_text: str
    genes: Dict[str, float]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Turn a financial life exploration run log into a plain-English explanation."
    )
    parser.add_argument("log_file", help="Path to a saved run log, usually captured with tee.")
    parser.add_argument(
        "-o",
        "--output",
        help="Optional output text file. If omitted, the report is only printed.",
    )
    return parser.parse_args()


def parse_log(path: Path) -> RunSummary:
    text = path.read_text(encoding="utf-8")

    config_match = re.search(r"^Scale config:\s+(.+)$", text, re.MULTILINE)
    if not config_match:
        raise ValueError("Could not find 'Scale config:' line in the log.")

    config = dict(re.findall(r"(\w+)=([^\s]+)", config_match.group(1)))

    best_block = re.search(
        r"- The best individual is:\s*"
        r"generation:\s*(\d+)\s*"
        r"index:\s*(\d+)\s*"
        r"fitness:\s*([0-9.]+)\s*"
        r"string:\s*([0-9.;-]+)",
        text,
        re.MULTILINE,
    )
    if not best_block:
        raise ValueError("Could not find the final best-individual block in the log.")

    genome_text = best_block.group(4)
    genome_values = [float(value) for value in genome_text.split(";")]
    if len(genome_values) != len(GENE_NAMES):
        raise ValueError(
            f"Expected {len(GENE_NAMES)} genes, found {len(genome_values)} in '{genome_text}'."
        )

    genes = dict(zip(GENE_NAMES, genome_values))
    return RunSummary(
        config=config,
        best_generation=int(best_block.group(1)),
        best_index=int(best_block.group(2)),
        best_fitness=float(best_block.group(3)),
        genome_text=genome_text,
        genes=genes,
    )


def describe_band(value: float, low: str, mid: str, high: str) -> str:
    if value < 0.34:
        return low
    if value < 0.67:
        return mid
    return high


def explain_genes(genes: Dict[str, float]) -> Dict[str, str]:
    return {
        "frugality": describe_band(
            genes["frugality"], "very low frugality", "moderate frugality", "high frugality"
        ),
        "savings_discipline": describe_band(
            genes["savings_discipline"],
            "weak savings discipline",
            "steady savings discipline",
            "strong savings discipline",
        ),
        "career_drive": describe_band(
            genes["career_drive"], "low career drive", "moderate career drive", "high career drive"
        ),
        "risk_tolerance": describe_band(
            genes["risk_tolerance"], "low market risk", "moderate market risk", "high market risk"
        ),
        "resilience": describe_band(
            genes["resilience"], "low resilience", "moderate resilience", "high resilience"
        ),
        "family_pressure": describe_band(
            genes["family_pressure"],
            "low family pressure",
            "moderate family pressure",
            "high family pressure",
        ),
        "education_investment": describe_band(
            genes["education_investment"],
            "low education investment",
            "moderate education investment",
            "high education investment",
        ),
    }


def likely_strengths(genes: Dict[str, float]) -> List[str]:
    strengths: List[str] = []

    if genes["savings_discipline"] >= 0.67:
        strengths.append("strong saving behavior likely helped retain income instead of letting cash leak into consumption")
    if genes["frugality"] >= 0.67:
        strengths.append("high frugality likely kept discretionary spending and debt growth under control")
    elif genes["frugality"] <= 0.20:
        strengths.append("very low frugality means this policy was not winning by austerity alone")

    if genes["career_drive"] >= 0.67:
        strengths.append("high career drive likely improved earnings through stronger income growth and promotions")
    elif genes["career_drive"] <= 0.20:
        strengths.append("low career drive suggests the policy won more by stability than by chasing income growth")

    if genes["risk_tolerance"] <= 0.20:
        strengths.append("low risk tolerance likely reduced exposure to volatile asset outcomes")
    elif genes["risk_tolerance"] >= 0.67:
        strengths.append("high risk tolerance likely aimed for stronger asset growth at the cost of more volatility")

    if genes["resilience"] >= 0.67:
        strengths.append("high resilience likely helped households absorb shocks without spiraling")
    elif genes["resilience"] <= 0.33:
        strengths.append("low resilience means the policy probably depended on avoiding trouble rather than recovering from it")

    if genes["family_pressure"] <= 0.33:
        strengths.append("low family pressure likely avoided extra household-expansion costs such as dependents and home purchases")
    elif genes["family_pressure"] >= 0.67:
        strengths.append("high family pressure means this policy was willing to carry more family and housing costs")

    if genes["education_investment"] >= 0.67:
        strengths.append("high education investment likely traded short-term cost for better early-career growth")
    elif genes["education_investment"] <= 0.33:
        strengths.append("low education investment suggests the policy did not rely heavily on training-driven gains")

    if not strengths:
        strengths.append("the winning policy looks balanced across the genome rather than extreme in one direction")

    return strengths


def build_report(summary: RunSummary) -> str:
    gene_descriptions = explain_genes(summary.genes)
    strengths = likely_strengths(summary.genes)

    profile = ", ".join(
        [
            gene_descriptions["frugality"],
            gene_descriptions["savings_discipline"],
            gene_descriptions["career_drive"],
            gene_descriptions["risk_tolerance"],
        ]
    )

    lines = [
        "Financial Run Interpretation",
        f"Best fitness: {summary.best_fitness:.6f} at generation {summary.best_generation}, index {summary.best_index}",
        "Scale config: " + ", ".join(f"{key}={value}" for key, value in summary.config.items()),
        f"Winning genome: {summary.genome_text}",
        "",
        f"Policy profile: {profile}",
        "Gene-by-gene reading:",
    ]

    for name in GENE_NAMES:
        lines.append(f"- {name}: {summary.genes[name]:.5f} -> {gene_descriptions[name]}")

    lines.append("")
    lines.append("Why this policy likely won:")
    for reason in strengths[:5]:
        lines.append(f"- {reason}")

    lines.append("")
    lines.append(
        "Demo takeaway: this run did not just find a lower fitness number; it found a policy profile with a readable financial strategy."
    )

    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    log_path = Path(args.log_file)

    if not log_path.exists():
        raise SystemExit(f"Log file not found: {log_path}")

    summary = parse_log(log_path)
    report = build_report(summary)
    print(report)

    if args.output:
        Path(args.output).write_text(report + "\n", encoding="utf-8")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
