// SPDX-License-Identifier: Apache-2.0

pub mod categorizer;
pub mod collector;
pub mod parser;
pub mod types;

use types::{AliasGroup, CollectError};

/// Collect all aliases from zsh.
pub fn load_aliases() -> Result<Vec<AliasGroup>, CollectError> {
    let raw_output = collector::collect_raw_aliases()?;
    let parsed = parser::parse_alias_lines(&raw_output);
    Ok(categorizer::categorize_aliases(parsed))
}
