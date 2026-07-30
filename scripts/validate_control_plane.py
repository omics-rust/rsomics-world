#!/usr/bin/env python3

from __future__ import annotations

import csv
import re
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def section(text: str, start: str, end: str | None = None) -> str:
    body = text.split(start, 1)[1]
    return body.split(end, 1)[0] if end else body


def table_rows(text: str) -> list[list[str]]:
    return [
        [cell.strip() for cell in line.strip().strip("|").split("|")]
        for line in text.splitlines()
        if line.startswith("| `rsomics-")
    ]


def allowlist() -> tuple[set[str], set[str]]:
    text = (ROOT / "docs/00-overview/registry-reset-keep.txt").read_text()
    products_text, foundations_text = text.split("# Public foundations", 1)
    products = set(re.findall(r"^rsomics-[a-z0-9-]+$", products_text, re.MULTILINE))
    foundations = set(
        re.findall(r"^rsomics-[a-z0-9-]+$", foundations_text, re.MULTILINE)
    )
    return products, foundations


def registry() -> tuple[dict[str, str], set[str], Counter[str]]:
    text = (ROOT / "REGISTRY.md").read_text()
    product_text = section(text, "## Product families", "## Public foundations")
    foundation_text = section(
        text, "## Public foundations", "## Temporary public dependency"
    )
    products = {row[0].strip("`"): row[1] for row in table_rows(product_text)}
    foundations = {row[0].strip("`") for row in table_rows(foundation_text)}
    summary_text = section(product_text, "Product status summary:", "`pilot` means")
    summary = Counter(
        {
            row[0]: int(row[1])
            for row in [
                [cell.strip() for cell in line.strip().strip("|").split("|")]
                for line in summary_text.splitlines()
                if re.match(r"^\| (live|repo-only|pilot|planned) \|", line)
            ]
        }
    )
    return products, foundations, summary


def dossier_counts() -> dict[str, int]:
    text = (ROOT / "docs/10-products/README.md").read_text()
    portfolio = section(text, "## Portfolio map", "Counts are generated")
    return {
        row[0].strip("`"): int(row[1])
        for row in table_rows(portfolio)
    }


def inventory() -> tuple[set[str], Counter[str]]:
    path = ROOT / "docs/00-overview/portfolio-inventory.tsv"
    with path.open(newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    names = {row["crate"] for row in rows}
    return names, Counter(row["target_kind_provisional"] for row in rows)


def consolidation_outputs() -> set[str]:
    path = ROOT / "docs/00-overview/portfolio-consolidation-outputs.txt"
    return {
        line
        for raw in path.read_text().splitlines()
        if (line := raw.strip()) and not line.startswith("#")
    }


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> None:
    products, foundations = allowlist()
    registry_products, registry_foundations, registry_summary = registry()
    dossiers = dossier_counts()
    inventory_names, inventory_kinds = inventory()
    outputs = consolidation_outputs()

    require(len(products) == 30, f"expected 30 products, found {len(products)}")
    require(len(foundations) == 9, f"expected 9 foundations, found {len(foundations)}")
    require(registry_products.keys() == products, "registry product set differs")
    require(registry_foundations == foundations, "registry foundation set differs")
    require(dossiers.keys() == products, "dossier product set differs")
    require(sum(dossiers.values()) == 424, "dossier candidate counts do not sum to 424")
    require(len(inventory_names) == 622, "historical inventory must contain 622 crates")
    require(inventory_kinds["product"] == 424, "inventory must contain 424 product candidates")
    require(inventory_kinds["capability-pool"] == 170, "inventory must contain 170 capability candidates")
    require(inventory_kinds["foundation"] == 28, "inventory must contain 28 foundation candidates")
    require(outputs.isdisjoint(inventory_names), "consolidation output leaked into inventory")
    require(
        Counter(registry_products.values()) == registry_summary,
        "registry status summary differs from product rows",
    )

    print("control plane is internally consistent")


if __name__ == "__main__":
    main()
