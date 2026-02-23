// SPDX-License-Identifier: Apache-2.0

use ratatui::style::{Color, Modifier, Style};

pub const SELECTED_BG: Color = Color::Rgb(60, 60, 100);
pub const SELECTED_FG: Color = Color::White;
pub const GROUP_FG: Color = Color::Rgb(120, 180, 255);
pub const ALIAS_NAME_FG: Color = Color::Rgb(200, 200, 100);
pub const ALIAS_CMD_FG: Color = Color::Rgb(160, 160, 160);
pub const BORDER_COLOR: Color = Color::Rgb(80, 80, 120);
pub const BORDER_FOCUSED: Color = Color::Rgb(120, 140, 255);
pub const SEARCH_FG: Color = Color::Rgb(255, 200, 100);
pub const HELP_FG: Color = Color::Rgb(120, 120, 140);
pub const DETAIL_LABEL_FG: Color = Color::Rgb(140, 140, 180);
pub const DETAIL_VALUE_FG: Color = Color::White;

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER_FOCUSED)
    } else {
        Style::default().fg(BORDER_COLOR)
    }
}

pub fn selected_style() -> Style {
    Style::default()
        .bg(SELECTED_BG)
        .fg(SELECTED_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn group_style() -> Style {
    Style::default().fg(GROUP_FG).add_modifier(Modifier::BOLD)
}

pub fn alias_name_style() -> Style {
    Style::default().fg(ALIAS_NAME_FG)
}

pub fn alias_cmd_style() -> Style {
    Style::default().fg(ALIAS_CMD_FG)
}
