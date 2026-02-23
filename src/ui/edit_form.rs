// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::theme;
use crate::app::{EditField, EditState};

pub fn render(frame: &mut Frame, area: Rect, state: &EditState) {
    let title = if state.is_new {
        " New Alias "
    } else {
        " Edit Alias "
    };

    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .title(title);

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let label = Style::default().fg(theme::DETAIL_LABEL_FG);
    let value = Style::default().fg(theme::DETAIL_VALUE_FG);
    let active = Style::default()
        .fg(theme::SEARCH_FG)
        .add_modifier(Modifier::BOLD);
    let cursor = "█";

    let name_active = state.active_field == EditField::Name;
    let cmd_active = state.active_field == EditField::Command;

    let name_style = if name_active { active } else { value };
    let cmd_style = if cmd_active { active } else { value };

    let name_cursor = if name_active { cursor } else { "" };
    let cmd_cursor = if cmd_active { cursor } else { "" };

    let name_label_style = if name_active { active } else { label };
    let cmd_label_style = if cmd_active { active } else { label };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name:    ", name_label_style),
            Span::styled(&state.name, name_style),
            Span::styled(name_cursor, Style::default().fg(theme::SEARCH_FG)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Command: ", cmd_label_style),
            Span::styled(&state.command, cmd_style),
            Span::styled(cmd_cursor, Style::default().fg(theme::SEARCH_FG)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", label),
            Span::styled("Tab", Style::default().fg(theme::SEARCH_FG)),
            Span::styled(":next field  ", Style::default().fg(theme::HELP_FG)),
            Span::styled("Enter", Style::default().fg(theme::SEARCH_FG)),
            Span::styled(":save  ", Style::default().fg(theme::HELP_FG)),
            Span::styled("Esc", Style::default().fg(theme::SEARCH_FG)),
            Span::styled(":cancel", Style::default().fg(theme::HELP_FG)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
