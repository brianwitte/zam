// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .title(" Help ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let ks = Style::default().fg(theme::SEARCH_FG);
    let ds = Style::default().fg(theme::DETAIL_VALUE_FG);
    let hs = Style::default().fg(theme::GROUP_FG);
    let dim = Style::default().fg(theme::HELP_FG);

    // Two-column layout: left column 22 chars, right column remainder
    let col = 24;

    let lines = vec![
        Line::from(""),
        two_headers("  Navigation", "Search", col, hs),
        two_row("  j / k    ", "up / down", "/          ", "open search", col, ks, ds),
        two_row("  g / G    ", "top / bottom", "Esc        ", "close search", col, ks, ds),
        two_row("  Ctrl+d/u ", "page down/up", "Ctrl+u     ", "clear query", col, ks, ds),
        two_row("  Enter    ", "toggle group", "=query     ", "search commands", col, ks, ds),
        two_row("  Tab      ", "switch panel", "@query     ", "search groups", col, ks, ds),
        Line::from(""),
        two_headers("  Aliases", "Edit Mode", col, hs),
        two_row("  a        ", "create new", "Tab        ", "switch field", col, ks, ds),
        two_row("  e        ", "edit selected", "Enter      ", "save", col, ks, ds),
        two_row("  d        ", "delete selected", "Esc        ", "cancel", col, ks, ds),
        two_row("  ", "", "Ctrl+u     ", "clear field", col, ks, ds),
        Line::from(""),
        one_header("  General", hs),
        two_row("  ?        ", "this help", "q          ", "quit", col, ks, ds),
        two_row("  Ctrl+c   ", "force quit", "", "", col, ks, ds),
        Line::from(""),
        Line::from(Span::styled("  Press any key to close", dim)),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn two_headers<'a>(left: &'a str, right: &'a str, col: usize, style: Style) -> Line<'a> {
    let pad = col.saturating_sub(left.len());
    Line::from(vec![
        Span::styled(left, style),
        Span::raw(" ".repeat(pad)),
        Span::styled(right, style),
    ])
}

fn one_header<'a>(text: &'a str, style: Style) -> Line<'a> {
    Line::from(Span::styled(text, style))
}

fn two_row<'a>(
    lk: &'a str,
    ld: &'a str,
    rk: &'a str,
    rd: &'a str,
    col: usize,
    ks: Style,
    ds: Style,
) -> Line<'a> {
    let left_len = lk.len() + ld.len();
    let pad = col.saturating_sub(left_len);
    Line::from(vec![
        Span::styled(lk, ks),
        Span::styled(ld, ds),
        Span::raw(" ".repeat(pad)),
        Span::styled(rk, ks),
        Span::styled(rd, ds),
    ])
}
