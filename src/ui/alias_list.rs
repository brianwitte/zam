// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::theme;
use crate::app::{ListItem, PanelFocus};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    items: &[ListItem],
    selected: usize,
    scroll_offset: usize,
    focus: PanelFocus,
) {
    let focused = focus == PanelFocus::List;
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .title(" Aliases ");

    let inner = border.inner(area);
    frame.render_widget(border, area);

    let visible_height = inner.height as usize;
    if items.is_empty() || visible_height == 0 {
        let msg = Paragraph::new("  No aliases found").style(Style::default().fg(theme::HELP_FG));
        frame.render_widget(msg, inner);
        return;
    }

    let end = (scroll_offset + visible_height).min(items.len());
    let lines: Vec<Line> = (scroll_offset..end)
        .map(|i| render_item(&items[i], i == selected, inner.width))
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_item(item: &ListItem, is_selected: bool, panel_width: u16) -> Line<'static> {
    match item {
        ListItem::GroupHeader {
            display_name,
            alias_count,
            collapsed,
            ..
        } => {
            let arrow = if *collapsed { "▸" } else { "▾" };
            let style = if is_selected {
                theme::selected_style()
            } else {
                theme::group_style()
            };
            Line::from(vec![
                Span::styled(format!(" {arrow} "), style),
                Span::styled(display_name.clone(), style),
                Span::styled(
                    format!(" ({alias_count})"),
                    Style::default().fg(theme::HELP_FG),
                ),
            ])
        }
        ListItem::AliasEntry { alias, .. } => {
            let name_style = if is_selected {
                theme::selected_style()
            } else {
                theme::alias_name_style()
            };
            let cmd_style = if is_selected {
                theme::selected_style()
            } else {
                theme::alias_cmd_style()
            };

            let max_cmd = panel_width.saturating_sub(12) as usize;
            let cmd_display = if alias.command.len() > max_cmd {
                format!("{}…", &alias.command[..max_cmd.saturating_sub(1)])
            } else {
                alias.command.clone()
            };

            Line::from(vec![
                Span::styled(format!("   {:<8}", alias.name), name_style),
                Span::styled(cmd_display, cmd_style),
            ])
        }
    }
}
