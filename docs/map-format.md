<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — map file format

**Status:** in development. This page specifies version 1 of the plain-text
file that holds a dependency map. The library reads and writes it; the CLI's
`export` command writes it and `open` reads it.

`save` encrypts this same JSON into the format specified in
[encrypted-format.md](encrypted-format.md), and `open` recognizes both.

> **The file is plain text and is not encrypted.** Anyone who obtains a copy can
> read every resource, route and coverage level in it. Use `save` for a map you
> keep, and `export` only when you need to read or process the JSON; never store
> passwords, recovery codes or a real personal map in this format.

## Example

```json
{
  "format": "opsecmap-map",
  "version": 1,
  "resources": [
    { "id": "email", "label": "Test email", "coverage": "complete" },
    { "id": "phone", "label": "Test phone", "coverage": "unknown" }
  ],
  "routes": [
    {
      "id": "email-login",
      "label": "Email login",
      "target": "email",
      "prerequisites": ["phone"]
    }
  ]
}
```

The example is compacted for readability. OpSecMap writes each field on its own
line, indented by two spaces.

## Fields

| Field | Type | Rule |
| :--- | :--- | :--- |
| `format` | string | Always `"opsecmap-map"`. |
| `version` | integer | Always `1` for this version of the format. |
| `resources` | array | In map order. Each element has `id`, `label` and `coverage`. |
| `coverage` | string | `"unknown"`, `"incomplete"` or `"complete"`. |
| `routes` | array | In map order. Each element has `id`, `label`, `target` and `prerequisites`. |
| `prerequisites` | array of strings | Not empty, no duplicates; each ID must name a resource in `resources`. |

## Rules

- Every field is required, including `"coverage": "unknown"`. Unknown fields
  are rejected, so a misspelled field name is an error rather than being
  silently ignored.
- Identifiers and labels follow the rules of the map model: neither may be empty
  or contain only whitespace, resource IDs and route IDs are each unique, and a
  route's `target` must name a resource.
- Every resource is added to the map before any route, so a route may refer to
  any resource in `resources`, wherever it appears in the list. OpSecMap writes
  `routes` after `resources`; when reading, the order of the fields does not
  matter.
- `format` and `version` are checked first. A file with another version is
  reported as unsupported, even if its other fields differ from version 1.
- The same map always produces the same bytes, so two saved maps can be
  compared with `diff`.
