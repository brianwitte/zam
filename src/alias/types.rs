// SPDX-License-Identifier: Apache-2.0

use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum CollectError {
    #[error("failed to run zsh: {source}")]
    SpawnFailed { source: std::io::Error },

    #[error("zsh exited with {status}: {stderr}")]
    ShellFailed {
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[error("invalid UTF-8 from zsh: {source}")]
    InvalidUtf8 {
        source: std::string::FromUtf8Error,
    },
}

/// Represents where an alias was defined.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AliasSource {
    OhMyZshPlugin(String),
    OhMyZshLib(String),
    OhMyZshCustom(String),
    Zshrc,
    ZamExtension(String),
    ZamCustom,
    ZamOverride(String),
    Unknown,
}

impl AliasSource {
    pub fn display_name(&self) -> String {
        match self {
            AliasSource::OhMyZshPlugin(name) => format!("oh-my-zsh plugin: {name}"),
            AliasSource::OhMyZshLib(name) => format!("oh-my-zsh lib: {name}"),
            AliasSource::OhMyZshCustom(name) => format!("oh-my-zsh custom: {name}"),
            AliasSource::Zshrc => "~/.zshrc".to_string(),
            AliasSource::ZamExtension(name) => format!("zam ext: {name}"),
            AliasSource::ZamCustom => "zam custom".to_string(),
            AliasSource::ZamOverride(slug) => format!("zam override: {slug}"),
            AliasSource::Unknown => "Unknown".to_string(),
        }
    }

    pub fn file_path(&self) -> Option<String> {
        match self {
            AliasSource::OhMyZshPlugin(name) => {
                Some(format!("~/.oh-my-zsh/plugins/{name}/{name}.plugin.zsh"))
            }
            AliasSource::OhMyZshLib(name) => Some(format!("~/.oh-my-zsh/lib/{name}")),
            AliasSource::OhMyZshCustom(name) => Some(format!("~/.oh-my-zsh/custom/{name}")),
            AliasSource::Zshrc => Some("~/.zshrc".to_string()),
            AliasSource::ZamExtension(name) => {
                Some(format!("~/.config/zam/aliases/{name}.zsh"))
            }
            AliasSource::ZamCustom => Some("~/.config/zam/aliases/custom.zsh".to_string()),
            AliasSource::ZamOverride(slug) => {
                Some(format!("~/.config/zam/overrides/{slug}.zsh"))
            }
            AliasSource::Unknown => None,
        }
    }

    /// Sort key so groups appear in a consistent order.
    pub fn sort_key(&self) -> (u8, String) {
        match self {
            AliasSource::OhMyZshLib(n) => (0, n.clone()),
            AliasSource::OhMyZshPlugin(n) => (1, n.clone()),
            AliasSource::OhMyZshCustom(n) => (2, n.clone()),
            AliasSource::Zshrc => (3, String::new()),
            AliasSource::ZamExtension(n) => (7, n.clone()),
            AliasSource::ZamCustom => (8, String::new()),
            AliasSource::ZamOverride(slug) => (9, slug.clone()),
            AliasSource::Unknown => (10, String::new()),
        }
    }
}

/// Errors from managed directory operations.
#[derive(Debug)]
#[allow(dead_code)]
pub enum ManagedError {
    CreateDir(std::io::Error),
    WriteFile(std::io::Error),
    ReadFile(std::io::Error),
    NotInitialized,
    EmptyName,
    InvalidName(String),
}

impl fmt::Display for ManagedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagedError::CreateDir(e) => write!(f, "failed to create directory: {e}"),
            ManagedError::WriteFile(e) => write!(f, "failed to write file: {e}"),
            ManagedError::ReadFile(e) => write!(f, "failed to read file: {e}"),
            ManagedError::NotInitialized => {
                write!(f, "not initialized — run `zam --init` first")
            }
            ManagedError::EmptyName => write!(f, "alias name cannot be empty"),
            ManagedError::InvalidName(name) => {
                write!(f, "invalid alias name: {name}")
            }
        }
    }
}

/// A single alias definition.
#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub command: String,
    pub source: AliasSource,
    pub description: Option<String>,
}

/// A group of aliases from the same source.
#[derive(Debug, Clone)]
pub struct AliasGroup {
    pub source: AliasSource,
    pub aliases: Vec<Alias>,
    pub collapsed: bool,
}
