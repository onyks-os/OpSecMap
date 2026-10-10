#!/usr/bin/env bash
# crates.io serves, for this version, exactly the .crate attached to the signed
# release: its registry checksum is the release asset's SHA-256.
#   verify-release.d/crates-io.sh <assets-dir> <tag>
set -euo pipefail

dir="$1" tag="$2"
version="${tag#v}"
case "$version" in *-*) echo "    pre-release: not published to crates.io"; exit 0 ;; esac

# Right after the Release run, the publication is usually still running; it
# triggers this verification again when it completes.
state="$(gh run list -R "$REPO" --workflow publish-crates.yml --branch "$tag" --limit 1 \
         --json status,conclusion --jq '.[0] | "\(.status) \(.conclusion)"' 2>/dev/null || true)"
case "$state" in
    "completed success"|"") ;;
    completed*) echo "FAIL: the crates.io publication of $tag ended in '${state#completed }'" >&2; exit 1 ;;
    *) echo "    publication still running ($state): checked again when it completes"; exit 0 ;;
esac

crate="$dir/opsecmap-${version}.crate"
[ -f "$crate" ] || { echo "FAIL: the release has no $(basename "$crate")" >&2; exit 1; }
# crates.io asks API clients for a User-Agent that identifies them.
checksum="$(curl -fsSL -A "opsecmap release verification (https://github.com/$REPO)" \
    "https://crates.io/api/v1/crates/opsecmap/${version}" \
    | python3 -c 'import json, sys; print(json.load(sys.stdin)["version"]["checksum"])')" \
    || { echo "FAIL: crates.io has no opsecmap ${version}" >&2; exit 1; }
local_checksum="$(sha256sum "$crate" | cut -d' ' -f1)"
[ "$checksum" = "$local_checksum" ] \
    || { echo "FAIL: crates.io serves $checksum, the signed release asset is $local_checksum" >&2; exit 1; }
echo "    OK  $(basename "$crate")"
