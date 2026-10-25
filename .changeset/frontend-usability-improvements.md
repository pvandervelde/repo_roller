---
"repo_roller_api": minor
"repo-roller-frontend": minor
---

The create-repository wizard's template picker (Step 1) is more usable, and the dark-mode
user menu bug is fixed.

**Full, expandable template descriptions**: template card descriptions are now clamped to two
lines on every screen size, with a **Show more** / **Show less** toggle beneath them to reveal
or re-collapse the full text in place, without affecting the card's selection state.

**Human-readable template names**: template cards (and the template search) now show the
display name configured in the template's `.reporoller/template.toml` (`name` field) instead of
the raw repository slug, falling back to the slug when the template author hasn't set one.

**All tags are now shown and searchable**: template cards now display every tag configured on
the template (previously the backend only forwarded the first tag as a single `category`
field, so additional tags never reached the UI). The template search box now matches against
any of a template's tags, in addition to its name, display name, and description.

**Dark-mode fix**: the user name/avatar menu in the top-right of the app header, and its
dropdown, now use the app's theme colour tokens in both light and dark mode. Previously the
menu rendered as black text on a dark background in dark mode, making the signed-in user's
name unreadable.

### API change: `category` replaced by `displayName` and `tags`

`GET /api/v1/orgs/{org}/templates` and `GET /api/v1/orgs/{org}/templates/{template}` no longer
return a `category` field (which only ever carried the first configured tag). Both endpoints
now return:

- `displayName` (`string`) — the template's human-readable name, falling back to the
  repository slug (the existing `name` field) when unset.
- `tags` (`string[]`) — the template's full tag list. This is always present (an empty array
  when the template has no tags), unlike `category`, which was omitted entirely when absent.

**Migration**: any external client reading `category` from either endpoint must switch to
reading `tags` (e.g. `tags[0]` for the old single-category behaviour, or `tags.join(", ")` to
show them all) and, if a human-readable name is wanted, start reading the new `displayName`
field instead of `name`.

This is a `minor` bump for `repo_roller_api`, matching this repo's existing convention for
pre-1.0 (`0.x`) releases: breaking changes to API response shape are signalled with a `minor`
bump rather than `major`, the same precedent set by the structured-logging change in the
observability work (default log output format changed from plain text to JSON — a comparably
breaking change for consumers — and was shipped as `minor`). `major` is reserved in this repo
for the `1.0.0` stability milestone, not for pre-1.0 breaking changes.

`repo-roller-frontend` gets a `minor` bump: four new, additive user-facing features (expandable
descriptions, display names, full tag display/search) plus a visual bug fix (dark-mode user
menu contrast), with no removal of any existing page, route, or capability.
