// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::theme;
use crate::alias::types::Alias;
use crate::app::PanelFocus;

pub fn render(frame: &mut Frame, area: Rect, alias: Option<&Alias>, focus: PanelFocus) {
    let focused = focus == PanelFocus::Detail;
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .title(" Details ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let Some(alias) = alias else {
        let msg = Paragraph::new("  Select an alias to view details")
            .style(Style::default().fg(theme::HELP_FG));
        frame.render_widget(msg, inner);
        return;
    };

    let label = Style::default().fg(theme::DETAIL_LABEL_FG);
    let value = Style::default().fg(theme::DETAIL_VALUE_FG);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Alias:   ", label),
            Span::styled(&alias.name, theme::alias_name_style()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  Command: ", label)]),
    ];

    // Wrap long commands across multiple lines
    let max_width = inner.width.saturating_sub(13) as usize;
    if max_width > 0 {
        let mut start = 0;
        while start < alias.command.len() {
            let end = (start + max_width).min(alias.command.len());
            lines.push(Line::from(vec![
                Span::styled("             ", label),
                Span::styled(alias.command[start..end].to_string(), value),
            ]));
            start = end;
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Source:   ", label),
        Span::styled(alias.source.display_name(), value),
    ]));

    if let Some(file_path) = alias.source.file_path() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  File:    ", label),
            Span::styled(file_path, value),
        ]));
    }

    if let Some(desc) = &alias.description {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("  Description:", label)]));
        lines.push(Line::from(vec![
            Span::styled("    ", label),
            Span::styled(desc.clone(), value),
        ]));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
