---
title: "CLI Reference"
description: "Installation, global flags, environment variables, and exit codes for the repo-roller CLI."
audience: "all"
type: "reference"
---

# CLI Reference

The `repo-roller` command-line interface creates and inspects GitHub repositories.

## Installation

Download the binary for your platform from the [releases page](https://github.com/pvandervelde/repo_roller/releases) or install from source:

```bash
cargo install repo-roller
```

Verify the installation:

```bash
repo-roller --version
```

## Subcommands

| Subcommand | Description |
|---|---|
| [`create`](create.md) | Create a new GitHub repository |
| [`template info`](template.md) | Display details about a template |
| [`template validate`](template.md) | Validate a template's structure and configuration |
| [`validate`](validate.md) | Validate the metadata repository configuration |
| [`make-template`](make-template.md) | Scaffold a new template repository |

## Global flags

| Flag | Description |
|---|---|
| `--help`, `-h` | Print help for the command or subcommand |
| `--version`, `-V` | Print the CLI version |

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `GITHUB_TOKEN` | Yes | GitHub App installation token or personal access token used for all API calls |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug`, `trace`. Default: `warn` |

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Command succeeded |
| `1` | Command failed (reason printed to stderr) |
| `2` | Invalid arguments |
