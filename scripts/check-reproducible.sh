#!/usr/bin/env bash
#
# Build every release artifact twice, from two independent copies of the tree,
# and fail unless both builds are identical byte for byte.
#
#   scripts/check-reproducible.sh [output-dir]
#
# A Sigstore signature proves an artifact came from the release workflow; it
# cannot prove the workflow built what the source says. If the same commit always
# builds to the same bytes, anyone can rebuild a release and compare — and a build
# machine that injected something is caught by the first person who does. This
# script is the precondition: it shows the build *is* deterministic before anyone
# relies on that.
#
# Both builds use the commit's timestamp (SOURCE_DATE_EPOCH), in different
# directories, a few seconds apart, so anything that reads the clock or embeds a
# path shows up. It builds the tracked files of the working tree: uncommitted
# edits to tracked files are included, new files must be `git add`ed first.
#
# REPRO_PREPARE is run in each copy before the build (e.g. `npm ci`), and
# REPRO_MAKE_ARGS is added to its make command line (e.g. the interpreter of the
# project's virtualenv); the module's per-language fragment sets them for
# `make check-reproducible`.

set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$(mktemp -d)}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$repo"

SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
export SOURCE_DATE_EPOCH
echo "==> SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH ($(date -u -d "@$SOURCE_DATE_EPOCH" +%FT%TZ))"

# File modes are part of an archive: a copy made under umask 027 would build a
# different sdist or tarball from one made under 022.
umask 022

for run in 1 2; do
    tree="$(mktemp -d)"
    echo "==> Build $run in $tree"
    git ls-files -z | tar --null -T - -cf - | tar -xf - -C "$tree"
    [ "$run" = 2 ] && sleep 2
    (
        cd "$tree"
        if [ -n "${REPRO_PREPARE:-}" ]; then
            bash -c "$REPRO_PREPARE"
        fi
        # The SBOM validator's virtualenv is shared: it validates, it is not built.
        # shellcheck disable=SC2086 # REPRO_MAKE_ARGS is a list of VAR=value words
        make --no-print-directory release-artifacts SBOM_VENV="$repo/.cache/sbom-venv" ${REPRO_MAKE_ARGS:-}
    ) > "$out/build-$run.log" 2>&1 || {
        tail -30 "$out/build-$run.log" >&2
        echo "==> Build $run failed; full log: $out/build-$run.log" >&2
        exit 1
    }
    rm -rf "${out:?}/$run"
    cp -r "$tree/dist" "$out/$run"
    rm -rf "$tree"
done

echo "==> Comparing"
status=0
for f in "$out"/1/*; do
    name="$(basename "$f")"
    if [ ! -f "$out/2/$name" ]; then
        echo "DIFFERENT  $name (missing from build 2)"
        status=1
    elif cmp -s "$f" "$out/2/$name"; then
        echo "identical  $name  $(sha256sum "$f" | cut -c1-16)"
    else
        echo "DIFFERENT  $name"
        status=1
        if command -v diffoscope >/dev/null; then
            diffoscope --text "$out/$name.diff.txt" "$f" "$out/2/$name" > /dev/null || true
            echo "           see $out/$name.diff.txt"
        fi
    fi
done
for f in "$out"/2/*; do
    [ -e "$out/1/$(basename "$f")" ] || { echo "DIFFERENT  $(basename "$f") (missing from build 1)"; status=1; }
done

if [ "$status" = 0 ]; then
    echo "==> Reproducible: both builds are identical ($out)"
else
    echo "==> NOT reproducible ($out)" >&2
fi
exit "$status"
