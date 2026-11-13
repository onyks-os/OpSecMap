# Reproducible Builds

There is no published OpSecMap release to compare against an independent build.
The following describes the intended checks in the repository; it is not a claim
that any published artifact has been reproduced.

The same commit is meant to build to the same bytes, on any machine with the same toolchain. That is
what lets anyone check a release independently: rebuild the tag, compare with what was published.

## How it is held in place

- Every timestamp a build tool writes comes from the commit (`SOURCE_DATE_EPOCH`, set by
  `make/release.mk` from `git log`), not from the clock.
- `make release-artifacts` starts from an empty `dist/`, and `SHA256SUMS` is written from a sorted
  file list.
- [`scripts/check-reproducible.sh`](../scripts/check-reproducible.sh) builds twice, from two
  copies of the tree in different directories a few seconds apart, and fails on any difference —
  in CI on every pull request and push to `main`, and weekly.
- On every tag, the [Reproducible build](../.github/workflows/reproducible.yml) workflow builds the
  tag again on a different runner and requires the result to match the `SHA256SUMS` that
  `release.yml` published. A mismatch there means the published artifacts are not what the source
  builds to, and is handled as a security incident under [SECURITY.md](../SECURITY.md).

## Checking locally

```bash
make check-reproducible            # two builds, compared; OUT=<dir> keeps them
```

With [diffoscope](https://diffoscope.org/) installed, every difference is explained in
`<dir>/<artifact>.diff.txt`.

A local rebuild is compared against your own second build, not against the release: a different
distribution can compress the same files differently (zlib and zlib-ng produce different bytes), so
the authoritative comparison with a published release is the one CI makes on the same runner image.

No published release has yet been checked against an independent rebuild, so
cross-machine byte-for-byte reproducibility is not established for OpSecMap.
