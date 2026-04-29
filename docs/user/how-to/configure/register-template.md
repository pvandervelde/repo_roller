---
title: "Register a template with the metadata repository"
description: "Make a template repository discoverable by RepoRoller by adding the reporoller-template GitHub topic."
audience: "platform-engineer"
type: "how-to"
---

# Register a template with the metadata repository

RepoRoller discovers template repositories by searching for the `reporoller-template` GitHub topic in your organisation. No file needs to be added to the metadata repository itself.

## Add the topic to the template repository

1. On GitHub, open the template repository (e.g. `https://github.com/myorg/rust-service`).
2. Click the gear icon (⚙) next to the **About** section on the right side of the page.
3. In the **Topics** field, type `reporoller-template`.
4. Press **Enter** or **Space** to confirm the tag.
5. Click **Save changes**.

The topic is now set. RepoRoller will include this repository in the template list on the next request.

## Verify the registration

Run the validate command to confirm the template is accessible and its configuration is valid:

```bash
repo-roller template validate --org myorg --template rust-service
```

After adding the topic you can also list available templates via the API:

```bash
curl -H "Authorization: Bearer ${TOKEN}" \
  https://reporoller.myorg.example/api/v1/orgs/myorg/templates
```

## Remove a template from discovery

To stop RepoRoller from offering a template, remove the `reporoller-template` topic from the repository. No other changes are required.

## Related guides

- [Validate the metadata repository configuration](validate-configuration.md)
- [Templates API](../../reference/api/templates.md)
