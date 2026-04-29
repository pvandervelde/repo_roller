---
title: "Handlebars syntax in templates"
description: "Quick reference for supported Handlebars expressions in template file content."
audience: "platform-engineer"
type: "reference"
---

# Handlebars syntax in templates

RepoRoller uses [Handlebars](https://handlebarsjs.com/) for variable substitution in template file content and file/directory names.

## Supported expressions

### Variable substitution

```handlebars
{{variable_name}}
```

Substitutes the value of `variable_name`. **Double braces HTML-encode special characters** (see warning below). Use triple braces to output the raw value without encoding:

```handlebars
{{{variable_name}}}
```

> **Warning — HTML encoding in non-HTML files**
>
> `{{variable}}` encodes the following characters before inserting them:
>
> | Character | Encoded as |
> |-----------|------------|
> | `<`  | `&lt;`   |
> | `>`  | `&gt;`   |
> | `&`  | `&amp;`  |
> | `"`  | `&quot;` |
> | `'`  | `&#x27;` |
> | `=`  | `&#x3D;` |
> | `` ` `` | `&#x60;` |
>
> This is correct and safe for **HTML or Markdown files** rendered in a browser.
>
> For **non-HTML files** — shell scripts, Makefiles, YAML, TOML, Rust source files — encoding will corrupt your content (`&&` becomes `&amp;&amp;`, backticks become `&#x60;`, etc.).
>
> **Use `{{{triple braces}}}` for all non-HTML file types.** This passes values through byte-for-byte with no encoding.

**Example — Markdown (double braces are fine):**

```markdown
# {{repo_name}}

Service URL: https://{{org_name}}.example.com/{{service_name}}
```

**Example — Shell script (use triple braces):**

```bash
#!/bin/bash
set -e
{{{build_command}}}
```

**Example — YAML (use triple braces):**

```yaml
command: {{{command}}}
args: {{{args}}}
```

---

### Conditional blocks

```handlebars
{{#if variable_name}}
  Content when variable is truthy (non-empty string, or boolean true)
{{else}}
  Content when variable is falsy (empty string, undefined, or boolean false)
{{/if}}
```

```handlebars
{{#unless variable_name}}
  Content when variable is falsy
{{/unless}}
```

**Example:**

```toml
# config.toml
{{#if enable_metrics}}
[metrics]
port = 9090
{{/if}}
```

---

### Iteration

```handlebars
{{#each list_variable}}
  Item index: {{@index}}
  Item value: {{this}}
{{/each}}
```

> **Note:** Iteration requires the variable to be declared as a list type in `[variables]`. Currently only built-in list expansion is supported; custom list variables are not yet supported.

---

### Escaping literal braces

To output a literal `{{` in the processed file, prefix with a backslash:

```handlebars
\{{not_a_variable}}
```

Produces:

```
{{not_a_variable}}
```

---

## Unsupported features

The following standard Handlebars features are **not supported**:

| Feature | Notes |
|---|---|
| Partials (`{{> partial-name}}`) | Not supported |
| Custom helpers | Not supported |
| Subexpressions (`{{helper (inner value)}}`) | Not supported |
| `@key` in object iteration | Not available |
| Block parameters (`as |item|`) | Not supported |

If your template content requires one of these features, consider pre-processing the files before using them as template source.

---

## Where substitution applies

| Location | Substitution applied? |
|---|---|
| Text file content | Yes |
| File names (including directories) | Yes |
| Binary file content | No |
| Files matching `exclude_patterns` | No |
| `.reporoller/` directory content | Never |
