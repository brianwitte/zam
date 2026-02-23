// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    display_query: &str,
    focused: bool,
    visible: usize,
    total: usize,
) {
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .title(" Search ");

    let cursor_char = if focused { "█" } else { "" };
    let count_text = if visible == total {
        format!("  ({total} aliases)")
    } else {
        format!("  ({visible}/{total} aliases)")
    };

    let empty = display_query.is_empty();
    let query_span = if empty {
        Span::styled("name  =command  @group", Style::default().fg(theme::HELP_FG))
    } else {
        Span::styled(
            display_query,
            Style::default()
                .fg(theme::SEARCH_FG)
                .add_modifier(Modifier::BOLD),
        )
    };

    let line = Line::from(vec![
        Span::styled("[/] ", Style::default().fg(theme::HELP_FG)),
        Span::styled(format!("{label}: "), Style::default().fg(theme::DETAIL_LABEL_FG)),
        query_span,
        Span::styled(cursor_char, Style::default().fg(theme::SEARCH_FG)),
        Span::styled(count_text, Style::default().fg(theme::HELP_FG)),
    ]);

    let paragraph = Paragraph::new(line).block(border);
    frame.render_widget(paragraph, area);
}
