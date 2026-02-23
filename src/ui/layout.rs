// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub search_bar: Rect,
    pub alias_list: Rect,
    pub detail_panel: Rect,
    pub help_bar: Rect,
}

pub fn build_layout(area: Rect) -> AppLayout {
    // Vertical: search(3) | main area | help(2)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    // Horizontal: list(45%) | detail(55%)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(vertical[1]);

    AppLayout {
        search_bar: vertical[0],
        alias_list: horizontal[0],
        detail_panel: horizontal[1],
        help_bar: vertical[2],
    }
}
