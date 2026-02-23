// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::types::{Alias, AliasGroup, AliasSource};

/// Given raw (name, command) pairs, categorize each alias by scanning source files.
pub fn categorize_aliases(raw: Vec<(String, String)>) -> Vec<AliasGroup> {
    let Some(home) = dirs::home_dir() else {
        return group_all_unknown(raw);
    };

    let mut attribution: HashMap<String, (AliasSource, Option<String>)> = HashMap::new();

    categorize_zsh(&home, &mut attribution);

    // Build aliases with attribution
    let mut source_map: HashMap<String, Vec<Alias>> = HashMap::new();
    for (name, command) in raw {
        let (source, description) = attribution
            .remove(&name)
            .unwrap_or((AliasSource::Unknown, None));
        let display = source.display_name();
        let alias = Alias {
            name,
            command,
            source,
            description,
        };
        source_map.entry(display).or_default().push(alias);
    }

    let mut groups: Vec<AliasGroup> = source_map
        .into_values()
        .map(|mut aliases| {
            aliases.sort_by(|a, b| a.name.cmp(&b.name));
            let source = aliases[0].source.clone();
            AliasGroup {
                source,
                aliases,
                collapsed: false,
            }
        })
        .collect();

    groups.sort_by(|a, b| a.source.sort_key().cmp(&b.source.sort_key()));
    groups
}

fn categorize_zsh(home: &Path, attribution: &mut HashMap<String, (AliasSource, Option<String>)>) {
    let omz_dir = home.join(".oh-my-zsh");
    let zshrc_path = home.join(".zshrc");

    // 1. oh-my-zsh lib files
    if omz_dir.join("lib").is_dir() {
        scan_dir_for_aliases(
            &omz_dir.join("lib"),
            "zsh",
            attribution,
            AliasSource::OhMyZshLib,
        );
    }

    // 2. Enabled plugins from .zshrc
    let enabled_plugins = parse_zsh_plugins(&zshrc_path);
    for plugin in &enabled_plugins {
        let plugin_file = omz_dir
            .join("plugins")
            .join(plugin)
            .join(format!("{plugin}.plugin.zsh"));
        if plugin_file.is_file() {
            scan_file_for_aliases(
                &plugin_file,
                attribution,
                AliasSource::OhMyZshPlugin(plugin.clone()),
            );
        }
    }

    // 3. oh-my-zsh custom directory
    if omz_dir.join("custom").is_dir() {
        scan_dir_for_aliases(
            &omz_dir.join("custom"),
            "zsh",
            attribution,
            AliasSource::OhMyZshCustom,
        );
    }

    // 4. ~/.zshrc itself
    if zshrc_path.is_file() {
        scan_file_for_aliases(&zshrc_path, attribution, AliasSource::Zshrc);
    }

    // 5. zam custom aliases
    let custom_path = home.join(".config/zam/aliases/custom.zsh");
    if custom_path.is_file() {
        scan_file_for_aliases(&custom_path, attribution, AliasSource::ZamCustom);
    }

    // 6. zam extension files (any .zsh in aliases/ that isn't custom.zsh)
    let aliases_dir = home.join(".config/zam/aliases");
    if aliases_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&aliases_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("zsh") {
                    continue;
                }
                let filename = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if filename == "custom" {
                    continue;
                }
                let source = AliasSource::ZamExtension(filename);
                scan_file_for_aliases(&path, attribution, source);
            }
        }
    }
}

fn group_all_unknown(raw: Vec<(String, String)>) -> Vec<AliasGroup> {
    let mut aliases: Vec<Alias> = raw
        .into_iter()
        .map(|(name, command)| Alias {
            name,
            command,
            source: AliasSource::Unknown,
            description: None,
        })
        .collect();
    aliases.sort_by(|a, b| a.name.cmp(&b.name));
    vec![AliasGroup {
        source: AliasSource::Unknown,
        aliases,
        collapsed: false,
    }]
}

/// Parse `plugins=(foo bar ...)` from a shell rc file.
fn parse_zsh_plugins(zshrc_path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(zshrc_path) else {
        return Vec::new();
    };

    let prefix1 = "plugins=(";
    let prefix2 = "plugins =(";

    let mut items = Vec::new();
    let mut inside = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with(prefix1) || trimmed.starts_with(prefix2) {
            inside = true;
            let after = match trimmed.find('(') {
                Some(pos) => &trimmed[pos + 1..],
                None => continue,
            };
            if let Some(close) = after.find(')') {
                items.extend(after[..close].split_whitespace().map(String::from));
                inside = false;
            } else {
                items.extend(after.split_whitespace().map(String::from));
            }
        } else if inside {
            if let Some(close) = trimmed.find(')') {
                items.extend(trimmed[..close].split_whitespace().map(String::from));
                inside = false;
            } else {
                items.extend(trimmed.split_whitespace().map(String::from));
            }
        }
    }
    items
}

/// Scan all files with the given extension in a directory for alias definitions.
fn scan_dir_for_aliases<F>(
    dir: &Path,
    extension: &str,
    attribution: &mut HashMap<String, (AliasSource, Option<String>)>,
    make_source: F,
) where
    F: Fn(String) -> AliasSource,
{
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let source = make_source(filename);
            scan_file_for_aliases(&path, attribution, source);
        }
    }
}

/// Scan a single file for `alias name=...` definitions and preceding comments.
#[allow(clippy::needless_pass_by_value)] // source is cloned per-alias inside the loop
fn scan_file_for_aliases(
    path: &Path,
    attribution: &mut HashMap<String, (AliasSource, Option<String>)>,
    source: AliasSource,
) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        for alias_fragment in extract_alias_fragments(line) {
            let rest = alias_fragment
                .strip_prefix("-g ")
                .unwrap_or(&alias_fragment);
            if let Some(eq_pos) = rest.find('=') {
                let alias_name = rest[..eq_pos].trim().to_string();
                if !alias_name.is_empty() && !alias_name.contains(' ') {
                    let description = extract_description(&lines, i);
                    attribution
                        .entry(alias_name)
                        .or_insert((source.clone(), description));
                }
            }
        }
    }
}

/// Extract the part after each `alias ` keyword in a line.
/// Handles lines like `&& alias gfa='...'` or `alias -g foo=bar`.
fn extract_alias_fragments(line: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_from = 0;
    let bytes = line.as_bytes();
    while let Some(pos) = line[search_from..].find("alias ") {
        let abs_pos = search_from + pos;
        let is_word_start = abs_pos == 0 || {
            let prev = bytes[abs_pos - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };
        if is_word_start {
            let after = &line[abs_pos + 6..];
            let after = after.trim_start();
            if !after.is_empty() {
                results.push(after.to_string());
            }
        }
        search_from = abs_pos + 6;
    }
    results
}

/// Extract a description from comments immediately preceding the alias line.
fn extract_description(lines: &[&str], alias_line: usize) -> Option<String> {
    let mut comment_lines = Vec::new();
    let mut j = alias_line;
    while j > 0 {
        j -= 1;
        let trimmed = lines[j].trim();
        if trimmed.starts_with('#') {
            let text = trimmed.trim_start_matches('#').trim();
            if !text.is_empty() {
                comment_lines.push(text.to_string());
            }
        } else {
            break;
        }
    }
    if comment_lines.is_empty() {
        None
    } else {
        comment_lines.reverse();
        Some(comment_lines.join(" "))
    }
}
