---
"repo_roller_api": minor
"config_manager": patch
---

Backend API logs are now structured JSON by default (set `LOG_FORMAT=pretty` to restore
human-readable output for local development), every HTTP request is assigned a correlation ID
that is attached to all log lines produced while handling it and returned to the client as an
`X-Request-Id` response header, and all log output now passes through an automated
sanitization layer that redacts sensitive field values (tokens, passwords, secrets, etc.) and
secret-shaped substrings embedded in free-text log messages before they reach stdout.

This is a `minor` bump for `repo_roller_api` rather than `patch`: the default log *output
format* changes for every deployment that doesn't set `LOG_FORMAT` explicitly (anything
parsing or grepping the previous plain-text log lines will need to adapt to JSON), and the new
`X-Request-Id` response header is additive but observable, user-facing API behavior. No
request/response body schema changed and no endpoint behavior changed, so this is not a
`major` bump.

`config_manager` gets a `patch`: an internal-only fix (a DEBUG-level log line in
`github_metadata_provider.rs` no longer dumps the full, untrusted `TemplateConfig` via
`{:?}`, logging a bounded summary of variable names instead) with no change to any public
API or observable behavior for consumers of the crate.
