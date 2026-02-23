// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::theme;
use crate::app::PendingAction;

pub fn render(frame: &mut Frame, area: Rect, pending: &PendingAction) {
    match pending {
        PendingAction::Delete {
            alias_name,
            alias_source,
        } => render_delete(frame, area, alias_name, &alias_source.display_name()),
        PendingAction::CollisionOverride {
            edit_state,
            collision_desc,
        } => render_collision(frame, area, &edit_state.name, collision_desc),
    }
}

fn render_delete(frame: &mut Frame, area: Rect, name: &str, source_display: &str) {
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .title(" Delete Alias ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let label = Style::default().fg(theme::DETAIL_LABEL_FG);
    let value = Style::default().fg(theme::DETAIL_VALUE_FG);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Delete alias ", label),
            Span::styled(name, theme::alias_name_style()),
            Span::styled("?", label),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Source: ", label),
            Span::styled(source_display, value),
        ]),
        Line::from(""),
        Line::from(""),
        confirm_keys_line(),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn render_collision(frame: &mut Frame, area: Rect, name: &str, collision_desc: &str) {
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .title(" Name Collision ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let label = Style::default().fg(theme::DETAIL_LABEL_FG);
    let value = Style::default().fg(theme::DETAIL_VALUE_FG);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  \"", label),
            Span::styled(name, theme::alias_name_style()),
            Span::styled("\" ", label),
            Span::styled(collision_desc, value),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Override and save?", label)),
        Line::from(""),
        Line::from(""),
        confirm_keys_line(),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn confirm_keys_line() -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("y", Style::default().fg(theme::SEARCH_FG)),
        Span::styled(":confirm  ", Style::default().fg(theme::HELP_FG)),
        Span::styled("n", Style::default().fg(theme::SEARCH_FG)),
        Span::styled("/", Style::default().fg(theme::HELP_FG)),
        Span::styled("Esc", Style::default().fg(theme::SEARCH_FG)),
        Span::styled(":cancel", Style::default().fg(theme::HELP_FG)),
    ])
}
