# Secrets in the repository

Do not commit passwords, recovery codes, tokens, private keys, or personal maps.
A map describes recovery material by name, never by its secret value, but even a
map with only names reveals how someone's accounts can be recovered. Test
fixtures, examples and issue reports must use fictional resources.

The program needs no credentials or environment variables to build or run.

`make lint-secrets` scans tracked files for a limited set of secret-like
patterns. That check cannot prove a repository is free of secrets. GitHub
account settings, including push protection, are outside what this source tree
can verify.

If a credential is exposed, revoke or rotate it promptly. Removing text from a
later commit does not erase an already published credential; coordinate any
history rewrite with contributors and follow [SECURITY.md](SECURITY.md) for
private reporting.
