#!/usr/bin/env bash
#
# Verify a published release the way a user would, from what is actually
# published — not from the files the release job had on disk.
#
#   scripts/verify-release.sh v1.2.3 [download-dir]
#
# Fails on the first thing that does not hold:
#   1. every asset has a Sigstore bundle, and every bundle has an asset;
#   2. every asset verifies against its bundle, and the signing certificate names
#      one of this repository's release workflows *at this tag* — a valid
#      signature made by any other workflow, fork, or ref is rejected;
#   3. SHA256SUMS matches every file it lists;
#   4. every hook in scripts/verify-release.d/ passes — the publishing modules
#      put one there to check that their registry serves the signed bytes.
#
# The workflows allowed to sign are release.yml, and package-*.yml for the
# distribution packages that are built in a container of their own.
#
# Needs: gh (authenticated, or GH_TOKEN), the sigstore CLI (pip install sigstore),
# sha256sum, and whatever the hooks need (curl, python3).

set -euo pipefail

tag="${1:?usage: $0 <tag, e.g. v1.2.3> [download-dir]}"
repo="${GITHUB_REPOSITORY:-onyks-it/OpSecMap}"
dir="${2:-$(mktemp -d)}"
issuer="https://token.actions.githubusercontent.com"
root="$(cd "$(dirname "$0")/.." && pwd)"

command -v sigstore >/dev/null || { echo "the sigstore CLI is required: pip install sigstore" >&2; exit 2; }

# The identities a release asset may carry: the workflow file and the tag.
identities=()
for wf in "$root"/.github/workflows/release.yml "$root"/.github/workflows/package-*.yml; do
    [ -f "$wf" ] || continue
    identities+=("https://github.com/${repo}/.github/workflows/$(basename "$wf")@refs/tags/${tag}")
done
[ "${#identities[@]}" -gt 0 ] || { echo "FAIL: no release workflow found under .github/workflows" >&2; exit 1; }

echo "==> Downloading the assets of $repo $tag into $dir"
mkdir -p "$dir"
gh release download "$tag" -R "$repo" -D "$dir" --clobber
cd "$dir"

shopt -s nullglob
assets=()
for f in *; do
    case "$f" in *.sigstore.json|*.sigstore) ;; *) assets+=("$f") ;; esac
done
[ "${#assets[@]}" -gt 0 ] || { echo "FAIL: the release has no assets" >&2; exit 1; }

bundle_of() {
    if [ -f "$1.sigstore.json" ]; then echo "$1.sigstore.json"
    elif [ -f "$1.sigstore" ]; then echo "$1.sigstore"
    fi
}

echo "==> 1. Every asset has a bundle, every bundle an asset"
for f in "${assets[@]}"; do
    [ -n "$(bundle_of "$f")" ] || { echo "FAIL: $f has no Sigstore bundle" >&2; exit 1; }
done
for b in *.sigstore.json *.sigstore; do
    a="${b%.json}"; a="${a%.sigstore}"
    [ -f "$a" ] || { echo "FAIL: bundle $b signs nothing in the release" >&2; exit 1; }
done
echo "    ${#assets[@]} assets, each with a bundle"

echo "==> 2. Signatures, bound to this repository's workflows at $tag"
for f in "${assets[@]}"; do
    signed_by=""
    for identity in "${identities[@]}"; do
        if sigstore verify identity --cert-identity "$identity" --cert-oidc-issuer "$issuer" \
            --bundle "$(bundle_of "$f")" "$f" >/dev/null 2>&1; then
            signed_by="${identity#*/.github/workflows/}"
            break
        fi
    done
    [ -n "$signed_by" ] || {
        echo "FAIL: $f is not signed by any of:" >&2
        printf '        %s\n' "${identities[@]}" >&2
        exit 1
    }
    echo "    OK  $f  (${signed_by%@*})"
done

echo "==> 3. SHA256SUMS"
if [ -f SHA256SUMS ]; then
    sha256sum --strict -c SHA256SUMS | sed 's/^/    /'
else
    echo "FAIL: the release has no SHA256SUMS" >&2
    exit 1
fi

echo "==> 4. Registries and other checks"
ran=0
for hook in "$root"/scripts/verify-release.d/*.sh; do
    [ -f "$hook" ] || continue
    echo "  -> $(basename "$hook")"
    REPO="$repo" bash "$hook" "$dir" "$tag"
    ran=$((ran + 1))
done
[ "$ran" -gt 0 ] || echo "    (no hooks in scripts/verify-release.d)"

echo "==> $repo $tag: verified"
