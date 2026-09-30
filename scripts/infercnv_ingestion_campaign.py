"""Private raw-creation metrics and closed-campaign acceptance; no launcher."""

from dataclasses import dataclass
import math
from pathlib import Path
import re
import statistics
from typing import Literal, Mapping, Sequence

from validate_infercnv_ingestion_measurement import validate_factory_results


InputCase = Literal["plain", "gzip"]
Implementation = Literal["rust", "infercnv"]
SECONDS = ("preparation_wall_seconds", "region_wall_seconds", "region_user_seconds",
           "region_system_seconds", "region_child_user_seconds", "region_child_system_seconds")
RSS = ("baseline_rss_bytes", "returned_rss_bytes")
KEYS = {"schema_version", "measurement", "implementation", "input_case", *SECONDS, *RSS}
NUMBER = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z")
INTEGER = re.compile(r"[0-9]+\Z")


@dataclass(frozen=True)
class RegionMetrics:
    schema_version: int
    measurement: str
    implementation: Implementation
    input_case: InputCase
    preparation_wall_seconds: float
    region_wall_seconds: float
    region_user_seconds: float
    region_system_seconds: float
    region_child_user_seconds: float
    region_child_system_seconds: float
    baseline_rss_bytes: int
    returned_rss_bytes: int


@dataclass(frozen=True)
class Slot:
    input_case: InputCase
    pair: int
    order: int
    pilot: bool
    implementation: Implementation


@dataclass(frozen=True)
class Sample:
    slot: Slot
    trial_dir: Path
    exit_status: int


@dataclass(frozen=True)
class CheckedTrial:
    slot: Slot
    trial_dir: Path
    metrics: RegionMetrics
    creation_checks: Mapping[str, dict]


@dataclass(frozen=True)
class Distribution:
    raw: tuple[float | int, ...]
    median: float | int
    minimum: float | int
    maximum: float | int


@dataclass(frozen=True)
class CaseSummary:
    input_case: InputCase
    rust: Mapping[str, Distribution]
    infercnv: Mapping[str, Distribution]
    strict_wall_advantage: bool
    signed_wall_gap_seconds: float
    wall_ranges_overlap: bool
    wall_overlap_seconds: float


@dataclass(frozen=True)
class CampaignSummary:
    trials: tuple[CheckedTrial, ...]
    cases: tuple[CaseSummary, ...]
    validated_results: int
    measured_samples: int
    strict_wall_advantage: bool


def _seconds(raw: str, key: str) -> float:
    if NUMBER.fullmatch(raw) is None:
        raise ValueError(f"invalid literal duration: {key}")
    value = float(raw)
    if value == 0 and any(digit in "123456789" for digit in raw.lower().split("e", 1)[0]):
        raise ValueError(f"nonzero duration underflows: {key}")
    if not math.isfinite(value) or value < 0 or key == "region_wall_seconds" and value == 0:
        raise ValueError(f"invalid finite duration: {key}")
    if key in ("region_child_user_seconds", "region_child_system_seconds") and value != 0:
        raise ValueError(f"unexpected child CPU: {key}")
    return value


def _rss(raw: str, key: str) -> int:
    if INTEGER.fullmatch(raw) is None:
        raise ValueError(f"invalid literal RSS: {key}")
    significant = raw.lstrip("0")
    if not significant or len(significant) > 20:
        raise ValueError(f"RSS outside positive u64: {key}")
    value = int(significant)
    if value > 2**64 - 1:
        raise ValueError(f"RSS outside positive u64: {key}")
    return value


def parse_region_metrics(text: str) -> RegionMetrics:
    """Parse literal LF TSV without CSV unquoting or whitespace normalization."""
    if type(text) is not str:
        raise ValueError("metrics must be literal text")
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    if not lines or lines[0] != "key\tvalue":
        raise ValueError("metrics header differs")
    values = {}
    for line in lines[1:]:
        fields = line.split("\t")
        if len(fields) != 2:
            raise ValueError("malformed metrics row")
        key, value = fields
        if key in values:
            raise ValueError(f"duplicate metrics key: {key}")
        values[key] = value
    if set(values) != KEYS:
        raise ValueError("metrics keys differ from exact twelve-field schema")
    if values["schema_version"] != "1" or values["measurement"] != "raw_creation":
        raise ValueError("metrics schema or measurement differs")
    if values["implementation"] not in ("rust", "infercnv") or values["input_case"] not in ("plain", "gzip"):
        raise ValueError("metrics implementation or case differs")
    numeric = {key: _seconds(values[key], key) for key in SECONDS}
    numeric.update({key: _rss(values[key], key) for key in RSS})
    return RegionMetrics(1, "raw_creation", values["implementation"], values["input_case"], **numeric)


def expected_schedule() -> tuple[Slot, ...]:
    return tuple(Slot(case, pair, order, pair == 0, implementation)
                 for case in ("plain", "gzip") for pair in range(8)
                 for order, implementation in enumerate(
                     ("rust", "infercnv") if pair == 0 or pair % 2 else ("infercnv", "rust"), 1))


def _check_schedule(samples: Sequence[Sample]) -> None:
    for sample in samples:
        if type(sample) is not Sample or type(sample.slot) is not Slot:
            raise ValueError("invalid campaign sample type")
        slot = sample.slot
        if type(slot.pair) is not int or type(slot.order) is not int or type(slot.pilot) is not bool or \
                type(slot.input_case) is not str or type(slot.implementation) is not str:
            raise ValueError("campaign slot field types differ")
        if type(sample.exit_status) is not int or sample.exit_status != 0:
            raise ValueError("failed or invalid process exit status")
        if not isinstance(sample.trial_dir, Path):
            raise ValueError("campaign trial directory must be a Path")
    if tuple(sample.slot for sample in samples) != expected_schedule():
        raise ValueError("missing, extra, duplicate or out-of-order campaign samples")
    directories = [sample.trial_dir.resolve(strict=True) for sample in samples]
    if len(set(directories)) != len(directories):
        raise ValueError("duplicate canonical campaign directory")
    for sample in samples:
        if any(path.is_symlink() for path in (sample.trial_dir, *sample.trial_dir.parents)):
            raise ValueError("symlink campaign directory path")


def _distribution(values: tuple[float | int, ...]) -> Distribution:
    return Distribution(values, statistics.median(values), min(values), max(values))


def _summarize_case(input_case: InputCase, trials: tuple[CheckedTrial, ...]) -> CaseSummary:
    groups = {}
    for implementation in ("rust", "infercnv"):
        measured = tuple(trial.metrics for trial in trials if trial.slot.input_case == input_case and
                         trial.slot.implementation == implementation and not trial.slot.pilot)
        groups[implementation] = {key: _distribution(tuple(getattr(metric, key) for metric in measured))
                                  for key in (*SECONDS, *RSS)}
    rust = groups["rust"]["region_wall_seconds"]
    infercnv = groups["infercnv"]["region_wall_seconds"]
    lower = max(rust.minimum, infercnv.minimum)
    upper = min(rust.maximum, infercnv.maximum)
    return CaseSummary(input_case, groups["rust"], groups["infercnv"], rust.maximum < infercnv.minimum,
                       infercnv.minimum - rust.maximum, lower <= upper, max(0., upper - lower))


def accept_campaign(samples: Sequence[Sample], full_dir: Path, maps_path: Path) -> CampaignSummary:
    """Check all declared processes and exact exports before any wall decision.

    Actual launch ordering/non-overlap and input/source/runtime authentication
    must be supplied by the later owning driver; this is not that gate.
    """
    samples = tuple(samples)
    _check_schedule(samples)
    checked = []
    for sample in samples:
        metrics_path = sample.trial_dir / "metrics.tsv"
        if metrics_path.is_symlink() or not metrics_path.is_file():
            raise ValueError(f"missing or nonregular metrics file: {metrics_path}")
        with metrics_path.open(encoding="utf-8", newline="") as source:
            metrics = parse_region_metrics(source.read())
        if (metrics.input_case, metrics.implementation) != (sample.slot.input_case, sample.slot.implementation):
            raise ValueError("metrics case or implementation differs from scheduled process")
        checks = validate_factory_results(sample.trial_dir, full_dir, maps_path)
        checked.append(CheckedTrial(sample.slot, sample.trial_dir, metrics, checks))
    trials = tuple(checked)
    cases = tuple(_summarize_case(case, trials) for case in ("plain", "gzip"))
    return CampaignSummary(trials, cases, sum(len(trial.creation_checks) for trial in trials),
                           sum(not trial.slot.pilot for trial in trials),
                           all(case.strict_wall_advantage for case in cases))
