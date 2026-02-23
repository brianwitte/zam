// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme;
use crate::app::InputMode;

const CELL: usize = 16;

type Bindings<'a> = Vec<(&'a str, &'a str)>;

pub fn render(frame: &mut Frame, area: Rect, mode: &InputMode, status: Option<&str>) {
    let (row1, row2): (Bindings, Bindings) = match mode {
        InputMode::Normal { .. } => (
            vec![
                ("/", "search"),
                ("a", "add"),
                ("e", "edit"),
                ("d", "delete"),
                ("?", "help"),
                ("q", "quit"),
            ],
            vec![
                ("j/k", "navigate"),
                ("g/G", "top/bottom"),
                ("^d/u", "page"),
                ("Enter", "toggle"),
                ("Tab", "focus"),
            ],
        ),
        InputMode::Search => (
            vec![
                ("Esc", "exit"),
                ("Enter", "confirm"),
                ("^u", "clear"),
            ],
            vec![
                ("=", "cmd search"),
                ("@", "group search"),
            ],
        ),
        InputMode::Editing { .. } => (
            vec![
                ("Tab", "next field"),
                ("Enter", "save"),
                ("Esc", "cancel"),
                ("^u", "clear field"),
            ],
            Vec::new(),
        ),
        InputMode::Confirm => (
            vec![
                ("y", "confirm"),
                ("n/Esc", "cancel"),
            ],
            Vec::new(),
        ),
        InputMode::ResultPopup => (
            vec![("any key", "continue")],
            Vec::new(),
        ),
        InputMode::Help => (
            vec![("any key", "close help")],
            Vec::new(),
        ),
    };

    let line1 = build_aligned_line(&row1);
    let mut line2 = build_aligned_line(&row2);

    if let Some(msg) = status {
        line2.push(Span::styled("  │ ", Style::default().fg(theme::HELP_FG)));
        line2.push(Span::styled(
            msg.to_string(),
            Style::default().fg(theme::SEARCH_FG),
        ));
    }

    let lines = vec![Line::from(line1), Line::from(line2)];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn build_aligned_line<'a>(bindings: &[(&'a str, &'a str)]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    for (key, desc) in bindings {
        spans.push(Span::styled(
            format!("{key}:"),
            Style::default().fg(theme::SEARCH_FG),
        ));
        let desc_width = CELL.saturating_sub(key.len() + 1);
        spans.push(Span::styled(
            format!("{:<width$}", desc, width = desc_width),
            Style::default().fg(theme::HELP_FG),
        ));
    }
    spans
}
