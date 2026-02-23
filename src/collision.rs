// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

use crate::alias::types::AliasGroup;

#[derive(Debug, Clone)]
pub enum Collision {
    Alias { source: String },
    Command,
    Builtin,
    Function,
}

impl Collision {
    pub fn description(&self) -> String {
        match self {
            Collision::Alias { source } => format!("alias already exists (source: {source})"),
            Collision::Command => "shadows a shell command".to_string(),
            Collision::Builtin => "shadows a shell builtin".to_string(),
            Collision::Function => "shadows a shell function".to_string(),
        }
    }
}

/// Check a single name for collisions against in-memory aliases and the shell.
pub fn check_name_collision(name: &str, groups: &[AliasGroup]) -> Option<Collision> {
    // Check in-memory aliases first
    for group in groups {
        for alias in &group.aliases {
            if alias.name == name {
                return Some(Collision::Alias {
                    source: group.source.display_name(),
                });
            }
        }
    }

    // Check shell (command, builtin, function)
    check_shell(name)
}

/// Check a batch of alias names for collisions (for extension install).
pub fn check_batch_collisions(
    aliases: &[(&str, &str, &str)],
    groups: &[AliasGroup],
) -> Vec<(String, Collision)> {
    let mut collisions = Vec::new();
    for &(name, _, _) in aliases {
        if let Some(c) = check_name_collision(name, groups) {
            collisions.push((name.to_string(), c));
        }
    }
    collisions
}

fn check_shell(name: &str) -> Option<Collision> {
    let output = Command::new("zsh")
        .arg("-c")
        .arg(format!("whence -w {name}"))
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();

    // Format: "name: type"
    let word_type = line.rsplit(": ").next()?;
    match word_type {
        "command" => Some(Collision::Command),
        "builtin" => Some(Collision::Builtin),
        "function" => Some(Collision::Function),
        _ => None,
    }
}
