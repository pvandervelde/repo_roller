# User Goals

## Primary Users

RepoRoller is a self-service tool for all members of a GitHub organization. All users share the
same role and the same capabilities — there is no admin tier in the web UI at this stage.

### User Types

**Active org member**
A developer, platform engineer, or other team member who belongs to the organization and needs to
create a new GitHub repository. Has access to the VPN. May or may not have used RepoRoller before.

**First-time user**
Same as above, but has never authenticated with the web UI. Must complete the sign-in flow before
reaching the creation form.

**Returning user**
Has a valid session from a previous visit. Arrives directly at the creation form.

---

## Goals by User Type

| User | Goal | Success Condition |
|---|---|---|
| First-time user | Sign in and create a repository | Repository exists on GitHub, creator's username in audit log, total time under 5 minutes |
| First-time user | Understand what templates are available before committing | Can browse templates in the creation wizard before finalising their choice |
| Returning user | Create a repository quickly | Arrives at the form immediately, no re-authentication required |
| Any user | Know whether their chosen repository name is valid and available | Real-time feedback while typing, before form submission |
| Any user | Understand what each template will produce | Template card shows enough context (description, type, tags) to make an informed choice |
| Any user | Understand what template variables are and what to enter | Each variable shows a description and, where applicable, a default value |
| Any user | Know the outcome of their creation request | Success: sees repository link. Failure: sees what went wrong and what to do next |

---

## Non-Goals (explicitly out of scope for this release)

- Browsing templates without creating a repository (no standalone template catalogue)
- Viewing or editing existing repository settings
- Managing organization configuration (templates, types, global defaults)
- Creating repositories in multiple organizations from one UI
- Admin functionality (validating org config, managing teams)
- Batch repository creation
