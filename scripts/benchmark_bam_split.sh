#!/usr/bin/env bash
set -euo pipefail

fixture=${1:?usage: benchmark_bam_split.sh FIXTURE_DIR OURS OUTPUT_DIR [RUNS]}
ours=${2:?usage: benchmark_bam_split.sh FIXTURE_DIR OURS OUTPUT_DIR [RUNS]}
output=${3:?usage: benchmark_bam_split.sh FIXTURE_DIR OURS OUTPUT_DIR [RUNS]}
runs=${4:-5}
rseqc=${RSOMICS_RSEQC_BIN:?RSOMICS_RSEQC_BIN must name the RSeQC bin directory}

[ -x "$ours" ]
[ -f "$fixture/rg-two.bam" ]
[ -f "$fixture/coordinate.bam" ]
[ -f "$fixture/genes.bed12" ]
[ ! -e "$output" ]
test "$(samtools --version | sed -n '1p')" = "samtools 1.24"
test "$($rseqc/split_bam.py --version 2>&1)" = "split_bam.py 5.0.4"
case ${TMPDIR:-} in
  "/Volumes/Zane's HDD/rsomics-tmp" | "/Volumes/Zane's HDD/rsomics-tmp/"*) ;;
  *) printf '%s\n' 'TMPDIR must use the approved external-disk scratch directory' >&2; exit 2 ;;
esac

mkdir "$output"
work=$(mktemp -d "$TMPDIR/bam-split-bench.XXXXXX")
trap 'find "$work" -depth -delete' EXIT

sha256() {
  shasum -a 256 "$1" | awk '{print $1}'
}

decoded_sha256() {
  samtools view --no-PG "$1" | shasum -a 256 | awk '{print $1}'
}

run_case() {
  mode=$1
  side=$2
  directory=$3
  mkdir "$directory"
  case "$mode:$side" in
    default:rsomics)
      "$ours" split --no-pg -@ 4 -b "$directory/out" "$fixture/rg-two.bam" >/dev/null ;;
    default:upstream)
      samtools split --no-PG -@ 4 -f "$directory/out.%!.bam" "$fixture/rg-two.bam" >/dev/null ;;
    parts:rsomics)
      "$ours" split --no-pg --parts 4 --seed 7 -b "$directory/out" "$fixture/coordinate.bam" >/dev/null ;;
    parts:upstream)
      "$rseqc/divide_bam.py" -i "$fixture/coordinate.bam" -n 4 -o "$directory/out" >/dev/null ;;
    genes:rsomics)
      "$ours" split --no-pg --genes "$fixture/genes.bed12" -b "$directory/out" "$fixture/coordinate.bam" >/dev/null ;;
    genes:upstream)
      "$rseqc/split_bam.py" -i "$fixture/coordinate.bam" -r "$fixture/genes.bed12" -o "$directory/out" >/dev/null ;;
    mates:rsomics)
      "$ours" split --no-pg --mates -b "$directory/out" "$fixture/coordinate.bam" >/dev/null ;;
    mates:upstream)
      "$rseqc/split_paired_bam.py" -i "$fixture/coordinate.bam" -o "$directory/out" >/dev/null ;;
  esac
}

record_file() {
  mode=$1
  side=$2
  label=$3
  path=$4
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$mode" "$side" "$label" "$(wc -c < "$path" | tr -d ' ')" \
    "$(sha256 "$path")" "$(decoded_sha256 "$path")" "$(samtools view -c "$path")" \
    >> "$output/outputs.tsv"
}

printf 'mode\tside\tlabel\tbytes\tsha256\tdecoded_sha256\trecords\n' > "$output/outputs.tsv"
for mode in default parts genes mates; do
  for side in rsomics upstream; do
    run_case "$mode" "$side" "$output/$mode-$side"
  done
done

for label in old new; do
  ours_path="$output/default-rsomics/out.$label.bam"
  upstream_path="$output/default-upstream/out.$label.bam"
  test "$(decoded_sha256 "$ours_path")" = "$(decoded_sha256 "$upstream_path")"
  record_file default rsomics "$label" "$ours_path"
  record_file default upstream "$label" "$upstream_path"
done
for label in 0 1 2 3; do
  record_file parts rsomics "$label" "$output/parts-rsomics/out.$label.bam"
  record_file parts upstream "$label" "$output/parts-upstream/out_$label.bam"
done
for mode in genes mates; do
  if [ "$mode" = genes ]; then labels='in ex junk'; else labels='R1 R2 unmap'; fi
  for label in $labels; do
    ours_path="$output/$mode-rsomics/out.$label.bam"
    upstream_path="$output/$mode-upstream/out.$label.bam"
    test "$(decoded_sha256 "$ours_path")" = "$(decoded_sha256 "$upstream_path")"
    record_file "$mode" rsomics "$label" "$ours_path"
    record_file "$mode" upstream "$label" "$upstream_path"
  done
done
test "$(awk -F '\t' '$1 == "parts" && $2 == "rsomics" {sum += $7} END {print sum}' "$output/outputs.tsv")" = 4000000
test "$(awk -F '\t' '$1 == "parts" && $2 == "upstream" {sum += $7} END {print sum}' "$output/outputs.tsv")" = 4000000

printf 'mode\ttrial\tside\treal_s\tuser_s\tsys_s\tmax_rss_bytes\n' > "$output/timings.tsv"
measure() {
  mode=$1
  trial=$2
  side=$3
  run_directory="$work/$mode-$trial-$side"
  timing="$work/time"
  /usr/bin/time -lp bash -c 'run_case "$@"' bash "$mode" "$side" "$run_directory" 2> "$timing"
  real=$(awk '$1 == "real" {print $2}' "$timing")
  user=$(awk '$1 == "user" {print $2}' "$timing")
  sys=$(awk '$1 == "sys" {print $2}' "$timing")
  rss=$(awk '/maximum resident set size/ {print $1}' "$timing")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$mode" "$trial" "$side" "$real" "$user" "$sys" "$rss" >> "$output/timings.tsv"
  find "$run_directory" -depth -delete
}
export -f run_case
export ours fixture rseqc
for mode in default parts genes mates; do
  for trial in $(seq 1 "$runs"); do
    if [ $((trial % 2)) -eq 1 ]; then
      measure "$mode" "$trial" rsomics
      measure "$mode" "$trial" upstream
    else
      measure "$mode" "$trial" upstream
      measure "$mode" "$trial" rsomics
    fi
  done
done

shasum -a 256 "$fixture/rg-two.bam" "$fixture/coordinate.bam" "$fixture/coordinate.bam.bai" "$fixture/genes.bed12" > "$output/fixtures.sha256"
sysctl -n machdep.cpu.brand_string > "$output/machine.txt"
uname -a >> "$output/machine.txt"
"${RUSTC:-rustc}" --version >> "$output/machine.txt"
samtools --version | sed -n '1,2p' >> "$output/machine.txt"
"$rseqc/split_bam.py" --version >> "$output/machine.txt" 2>&1
