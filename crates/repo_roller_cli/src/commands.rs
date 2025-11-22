//! Command modules for the RepoRoller CLI.
//!
//! This module contains all the command implementations for the CLI application.
//! Each submodule handles a specific command category:
//!
//! - `auth_cmd`: Authentication-related commands for GitHub credentials
//! - `config_cmd`: Configuration management commands for settings and templates
//! - `create_cmd`: Repository creation commands from templates
//! - `org_settings_cmd`: Organization settings inspection commands
//! - `template_cmd`: Template inspection and validation commands

pub mod auth_cmd;
pub mod config_cmd;
pub mod create_cmd;
pub mod org_settings_cmd;
pub mod template_cmd;
