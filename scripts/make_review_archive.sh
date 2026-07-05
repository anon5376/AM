#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/am001-b06-review.zip}"
manifest="${2:-artifacts/review_archive_manifest.txt}"

mkdir -p "$(dirname "$out")" "$(dirname "$manifest")"

tmp_files="$(mktemp)"
tmp_manifest="$(mktemp)"
trap 'rm -f "$tmp_files" "$tmp_manifest"' EXIT

git ls-files > "$tmp_files"

grep -Ev '(^target/|^\.git/|(^|/)\.DS_Store$)' "$tmp_files" > "$tmp_manifest"
mv "$tmp_manifest" "$tmp_files"

grep -vxF "$out" "$tmp_files" > "$tmp_manifest"
mv "$tmp_manifest" "$tmp_files"

if ! grep -qxF "$manifest" "$tmp_files"; then
  printf '%s\n' "$manifest" >> "$tmp_files"
fi

{
  printf 'commit %s\n' "$(git rev-parse HEAD)"
  printf 'archive %s\n' "$out"
  printf 'manifest %s\n' "$manifest"
  printf '\nincluded_top_level_paths:\n'
  awk -F/ '{print $1}' "$tmp_files" | sort -u
  printf '\nartifact_subpaths:\n'
  grep '^artifacts/' "$tmp_files" | sort || true
} > "$manifest"

rm -f "$out"
zip -q -X "$out" -@ < "$tmp_files"

printf 'archive=%s\n' "$out"
printf 'manifest=%s\n' "$manifest"
