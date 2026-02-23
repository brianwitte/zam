// SPDX-License-Identifier: Apache-2.0

pub mod alias_list;
pub mod confirm_dialog;
pub mod detail_panel;
pub mod edit_form;
pub mod help_bar;
pub mod help_panel;
pub mod layout;
pub mod result_popup;
pub mod search_bar;
pub mod theme;

use ratatui::Frame;

use crate::app::{AppState, InputMode};

pub fn render(frame: &mut Frame, app: &mut AppState) {
    let layout = layout::build_layout(frame.area());

    let list_inner_height = layout.alias_list.height.saturating_sub(2) as usize;
    app.compute_scroll(list_inner_height);

    let panel_focus = app.mode.panel_focus();

    search_bar::render(
        frame,
        layout.search_bar,
        app.search_field(),
        app.search_display(),
        app.mode.is_search(),
        app.visible_alias_count(),
        app.total_aliases(),
    );

    alias_list::render(
        frame,
        layout.alias_list,
        &app.visible_items,
        app.selected_index,
        app.list_scroll_offset,
        panel_focus,
    );

    // Detail panel branches on mode
    match app.mode {
        InputMode::Editing { .. } => {
            if let Some(state) = &app.edit_state {
                edit_form::render(frame, layout.detail_panel, state);
            }
        }
        InputMode::Confirm => {
            if let Some(pending) = &app.pending_action {
                confirm_dialog::render(frame, layout.detail_panel, pending);
            }
        }
        InputMode::Help => {
            help_panel::render(frame, layout.detail_panel);
        }
        _ => {
            detail_panel::render(
                frame,
                layout.detail_panel,
                app.selected_alias(),
                panel_focus,
            );
        }
    }

    help_bar::render(frame, layout.help_bar, &app.mode, app.status_message.as_deref());

    if let Some(msg) = &app.result_message {
        result_popup::render(frame, msg);
    }
}
