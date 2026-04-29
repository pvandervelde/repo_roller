---
title: "How repository visibility is determined"
description: "How the final repository visibility is determined from organisation policy and user request."
audience: "repository-creator"
type: "explanation"
---

# How repository visibility is determined

A repository's visibility (`private`, `public`, or `internal`) is not simply what the user requests. It is the result of a policy cascade.

## The cascade

RepoRoller determines visibility in the following order:

1. **Organisation default**: If the global configuration does not allow override, the default applies to all repositories regardless of what the user requests.

2. **Repository type restriction**: A repository type configuration may restrict or default the visibility for repositories of that type.

3. **User request**: If neither of the above has locked the visibility, the user's requested value is used.

In most organisations, the global default is `private` with `override_allowed = true`, meaning users can request `public` but the default is `private`.

## When a user cannot change visibility

If the global configuration has:

```toml
[repository]
visibility = { value = "private", override_allowed = false }
```

Any request specifying `visibility: "public"` results in an error. The repository is not created. The error message explains that visibility policy is locked.

Similarly, a repository type may lock visibility to `internal` so that all repositories of that type are internal regardless of the creation request.

## The `internal` visibility option

`internal` visibility is only available on GitHub Enterprise Cloud and GitHub Enterprise Server. On GitHub.com, only `private` and `public` are supported. Specifying `internal` on a platform that does not support it results in a GitHub API error.

## What users see in the web UI

The visibility field in the creation wizard is:

- **Shown with all options** when the user is permitted to choose
- **Pre-filled and read-only** when the organisation policy sets a fixed value
- **Hidden entirely** when the template's `repository_type` policies make visibility irrelevant to the user

This prevents confusion from presenting choices that would be rejected at submission time.
