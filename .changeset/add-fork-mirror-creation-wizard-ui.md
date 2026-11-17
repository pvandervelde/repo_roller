---
"repo-roller-frontend": minor
---

The create-repository wizard now supports creating a GitHub **fork** or a **mirror** of an
existing repository, alongside the existing template-based flow.

**Creation mode choice**: Step 1 of the wizard ("Choose a template") now shows a **Creation
mode** choice — Template, Fork, Mirror — but only once, and only offering the modes the
organisation has enabled (discovered via `GET /orgs/:org/creation-capabilities` when the wizard
loads). If neither Fork nor Mirror is enabled for the organisation, this choice does not appear
at all and the wizard behaves exactly as before.

**Source repository and history preservation**: choosing Fork or Mirror replaces the template
grid with a **Source repository** field (`owner/repo`). For Mirror, if the organisation has
enabled both ways of mirroring, a **History preservation** choice also appears ("Preserve full
history" vs. "Contents only (via pull request)"); if only one way is enabled, it's used
automatically with no extra control shown. Fork and Mirror never produce a Variables step.

**Designated template and source allow-list, surfaced in the UI**: if the organisation has
designated a settings-only template for the chosen mode, its name is shown as read-only
information on the Review step — it is never user-selectable for Fork/Mirror, matching the
backend's auto-discovery design. If the organisation has curated an allow-list of permitted
source owners and the typed owner isn't on it, a non-blocking advisory note is shown; the server
remains the actual enforcement point.

**Clear, specific rejection messages**: if the server rejects the request
(`CapabilityDisabled`, `SourceOwnerNotAllowed`, `TemplateNotDesignated`, or —
from the capability-discovery call itself — `AmbiguousTemplateDesignation`), the wizard now shows
a clear, specific inline error on the Review step and preserves every value entered so far,
instead of falling back to a generic error.

**Bug fix**: also fixed a pre-existing issue where selecting a repository type during standard
(template-based) creation caused the entire creation request to be silently rejected by the
backend, due to a `repository_type`/`repositoryType` JSON key-casing mismatch. Repository type
selection now works correctly for all creation modes.

A new shared `RadioGroup` component backs both the Creation mode and History preservation
choices.
