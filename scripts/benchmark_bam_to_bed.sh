#!/usr/bin/env bash
set -euo pipefail

fixture=${1:?usage: benchmark_bam_to_bed.sh FIXTURE OURS OUTPUT [RUNS]}
ours=${2:?usage: benchmark_bam_to_bed.sh FIXTURE OURS OUTPUT [RUNS]}
output=${3:?usage: benchmark_bam_to_bed.sh FIXTURE OURS OUTPUT [RUNS]}
runs=${4:-10}
bedtools=${BEDTOOLS:-bedtools}

[ -f "$fixture" ]
[ -x "$ours" ]
[ ! -e "$output" ]
test "$($bedtools --version)" = "bedtools v2.31.1"
case ${TMPDIR:-} in
  /Volumes/KIOXIA/Developments/tmp | /Volumes/KIOXIA/Developments/tmp/* | \
    "/Volumes/Zane's HDD/rsomics-tmp" | "/Volumes/Zane's HDD/rsomics-tmp/"*) ;;
  *) printf '%s\n' 'TMPDIR must use an approved external-disk scratch directory' >&2; exit 2 ;;
esac

work=$(mktemp -d "$TMPDIR/bam-to-bed-bench.XXXXXX")
trap 'rm -rf "$work"' EXIT

measure() {
  mode=$1
  trial=$2
  side=$3
  shift 3
  timing="$work/timing"
  /usr/bin/time -lp "$@" > /dev/null 2> "$timing"
  real=$(awk '$1 == "real" { print $2 }' "$timing")
  user=$(awk '$1 == "user" { print $2 }' "$timing")
  sys=$(awk '$1 == "sys" { print $2 }' "$timing")
  rss=$(awk '/maximum resident set size/ { print $1 }' "$timing")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$mode" "$trial" "$side" "$real" "$user" "$sys" "$rss" >> "$output"
}

measure_ours() {
  if [ -n "$ours_arg" ]; then
    measure "$1" "$2" rsomics "$ours" to-bed "$ours_arg" "$fixture"
  else
    measure "$1" "$2" rsomics "$ours" to-bed "$fixture"
  fi
}

measure_bedtools() {
  if [ -n "$bedtools_arg" ]; then
    measure "$1" "$2" bedtools "$bedtools" bamtobed "$bedtools_arg" -i "$fixture"
  else
    measure "$1" "$2" bedtools "$bedtools" bamtobed -i "$fixture"
  fi
}

printf 'mode\ttrial\tside\treal_s\tuser_s\tsys_s\tmax_rss_bytes\n' > "$output"
for mode in default split bed12 bedpe; do
  case "$mode" in
    default) ours_arg=; bedtools_arg= ;;
    split) ours_arg=--split; bedtools_arg=-split ;;
    bed12) ours_arg=--bed12; bedtools_arg=-bed12 ;;
    bedpe) ours_arg=--bedpe; bedtools_arg=-bedpe ;;
  esac
  if [ -n "$ours_arg" ]; then
    "$ours" to-bed "$ours_arg" "$fixture" > /dev/null
    "$bedtools" bamtobed "$bedtools_arg" -i "$fixture" > /dev/null
  else
    "$ours" to-bed "$fixture" > /dev/null
    "$bedtools" bamtobed -i "$fixture" > /dev/null
  fi
  for trial in $(seq 1 "$runs"); do
    if [ $((trial % 2)) -eq 1 ]; then
      measure_ours "$mode" "$trial"
      measure_bedtools "$mode" "$trial"
    else
      measure_bedtools "$mode" "$trial"
      measure_ours "$mode" "$trial"
    fi
  done
done
