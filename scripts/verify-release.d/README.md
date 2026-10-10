# Release verification hooks

`scripts/verify-release.sh` runs every `*.sh` here after it has checked the signatures and
`SHA256SUMS`, as:

```bash
REPO=<owner>/<repo> bash <hook> <directory holding the downloaded assets> <tag>
```

A hook fails the verification by exiting non-zero. The publishing modules each add one that checks
their registry serves exactly the bytes attached to the signed release, since an index that serves
something else under the same version is the failure a signature on the GitHub Release cannot see.
