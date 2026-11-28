---
"repo_roller_api": patch
"repo_roller_core": patch
---

Enforce the template `[repository_type]` policy server-side during repository creation (#288).

A creation request that supplies `repository_type` for a template with `policy = "fixed"` is now rejected with `400` (`InvalidConfiguration`), even when the value equals the template type. With `policy = "preferable"` the override is accepted and otherwise the template type is used. An invalid `repository_type` name is rejected with a validation error. Previously the field was silently ignored, so the policy was only enforced by the frontend.

Documentation: `[template].author` and `[template].tags` are required by the template parser; the template manifest example now matches `template-config.md`.
