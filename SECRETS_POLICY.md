# Secrets in the repository

Do not commit passwords, recovery codes, tokens, private keys, or personal maps.
The current CLI uses only fixed example labels and does not need application
credentials. The planned dependency map describes recovery material by name; it
is not a place to store the material's secret values.

`make lint-secrets` scans tracked files for a limited set of secret-like
patterns. That check cannot prove a repository is free of secrets. GitHub
account settings, including push protection, are outside what this source tree
can verify.

If a credential is exposed, revoke or rotate it promptly. Removing text from a
later commit does not erase an already published credential; coordinate any
history rewrite with contributors and follow [SECURITY.md](SECURITY.md) for
private reporting.
