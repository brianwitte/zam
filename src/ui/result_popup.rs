// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use super::theme;

pub fn render(frame: &mut Frame, message: &str) {
    let area = centered_rect(50, 7, frame.area());

    frame.render_widget(Clear, area);

    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .title(" Result ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let label = Style::default().fg(theme::DETAIL_LABEL_FG);
    let value = Style::default().fg(theme::DETAIL_VALUE_FG);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {message}"), value)),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("  Press any key to continue", label)),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}
