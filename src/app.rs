// SPDX-License-Identifier: Apache-2.0

use crate::action::Action;
use crate::alias::types::{Alias, AliasGroup, AliasSource};
use crate::collision;
use crate::managed;

enum SearchField {
    Name,
    Command,
    Group,
}

/// Which panel has focus when in normal mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    List,
    Detail,
}

/// Which field is active in edit mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Name,
    Command,
}

/// The current input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal { focus: PanelFocus },
    Search,
    Editing { field: EditField },
    Confirm,
    ResultPopup,
    Help,
}

impl InputMode {
    pub fn is_search(self) -> bool {
        matches!(self, InputMode::Search)
    }

    pub fn panel_focus(self) -> PanelFocus {
        match self {
            InputMode::Normal { focus } => focus,
            _ => PanelFocus::List,
        }
    }
}

/// State for the edit form (create or edit).
#[derive(Debug, Clone)]
pub struct EditState {
    pub name: String,
    pub command: String,
    pub active_field: EditField,
    pub is_new: bool,
    /// For edits: the original name and source.
    pub original_name: Option<String>,
    pub original_source: Option<AliasSource>,
}

/// Pending action that needs confirmation.
#[derive(Debug, Clone)]
pub enum PendingAction {
    Delete {
        alias_name: String,
        alias_source: AliasSource,
    },
    CollisionOverride {
        edit_state: EditState,
        collision_desc: String,
    },
}

/// Represents a visible item in the left-panel list.
#[derive(Debug, Clone)]
pub enum ListItem {
    GroupHeader {
        group_index: usize,
        display_name: String,
        alias_count: usize,
        collapsed: bool,
    },
    AliasEntry {
        group_index: usize,
        alias: Alias,
    },
}

pub struct AppState {
    pub groups: Vec<AliasGroup>,
    pub visible_items: Vec<ListItem>,
    pub selected_index: usize,
    pub search_query: String,
    pub mode: InputMode,
    pub should_quit: bool,
    pub list_scroll_offset: usize,
    pub edit_state: Option<EditState>,
    pub pending_action: Option<PendingAction>,
    pub result_message: Option<String>,
    pub status_message: Option<String>,
    pub status_tick: u8,
}

impl AppState {
    pub fn new(groups: Vec<AliasGroup>) -> Self {
        let mut state = AppState {
            groups,
            visible_items: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            mode: InputMode::Normal {
                focus: PanelFocus::List,
            },
            should_quit: false,
            list_scroll_offset: 0,
            edit_state: None,
            pending_action: None,
            result_message: None,
            status_message: None,
            status_tick: 0,
        };
        state.rebuild_visible();
        state
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ScrollUp => self.move_selection(-1),
            Action::ScrollDown => self.move_selection(1),
            Action::PageUp => self.move_selection(-10),
            Action::PageDown => self.move_selection(10),
            Action::GoToTop => self.selected_index = 0,
            Action::GoToBottom => {
                if !self.visible_items.is_empty() {
                    self.selected_index = self.visible_items.len() - 1;
                }
            }
            Action::ToggleCollapse => self.toggle_selected_group(),
            Action::FocusSearch => self.mode = InputMode::Search,
            Action::ExitSearch => {
                self.mode = InputMode::Normal {
                    focus: PanelFocus::List,
                };
            }
            Action::SwitchFocus => {
                let new_focus = match self.mode.panel_focus() {
                    PanelFocus::List => PanelFocus::Detail,
                    PanelFocus::Detail => PanelFocus::List,
                };
                self.mode = InputMode::Normal { focus: new_focus };
            }
            Action::SearchInput(c) => {
                self.search_query.push(c);
                self.rebuild_visible();
                self.selected_index = 0;
            }
            Action::SearchBackspace => {
                self.search_query.pop();
                self.rebuild_visible();
                self.selected_index = 0;
            }
            Action::SearchClear => {
                self.search_query.clear();
                self.rebuild_visible();
                self.selected_index = 0;
            }

            // CRUD
            Action::CreateAlias => self.start_create(),
            Action::EditAlias => self.start_edit(),
            Action::DeleteAlias => self.start_delete(),

            // Edit mode
            Action::EditInput(c) => self.edit_input(c),
            Action::EditBackspace => self.edit_backspace(),
            Action::EditClear => self.edit_clear(),
            Action::EditNextField | Action::EditPrevField => self.edit_toggle_field(),
            Action::SaveEdit => self.save_edit(),
            Action::CancelEdit => self.cancel_edit(),

            // Confirm mode
            Action::ConfirmYes => self.execute_pending(),
            Action::ConfirmNo => self.cancel_confirm(),

            // Result popup
            Action::DismissResult => {
                self.result_message = None;
                self.mode = InputMode::Normal {
                    focus: PanelFocus::List,
                };
            }

            // Help
            Action::ShowHelp => self.mode = InputMode::Help,
            Action::CloseHelp => {
                self.mode = InputMode::Normal {
                    focus: PanelFocus::List,
                };
            }
        }
        self.clamp_selection();
    }

    // ── CRUD helpers ──────────────────────────────────────────────────

    fn start_create(&mut self) {
        if !managed::is_initialized() {
            self.set_status("Not initialized — run `zam --init` first");
            return;
        }
        self.edit_state = Some(EditState {
            name: String::new(),
            command: String::new(),
            active_field: EditField::Name,
            is_new: true,
            original_name: None,
            original_source: None,
        });
        self.mode = InputMode::Editing {
            field: EditField::Name,
        };
    }

    fn start_edit(&mut self) {
        if !managed::is_initialized() {
            self.set_status("Not initialized — run `zam --init` first");
            return;
        }
        let Some(alias) = self.selected_alias().cloned() else {
            return;
        };
        self.edit_state = Some(EditState {
            name: alias.name.clone(),
            command: alias.command.clone(),
            active_field: EditField::Command,
            is_new: false,
            original_name: Some(alias.name),
            original_source: Some(alias.source),
        });
        self.mode = InputMode::Editing {
            field: EditField::Command,
        };
    }

    fn start_delete(&mut self) {
        if !managed::is_initialized() {
            self.set_status("Not initialized — run `zam --init` first");
            return;
        }
        let Some(alias) = self.selected_alias().cloned() else {
            return;
        };
        self.pending_action = Some(PendingAction::Delete {
            alias_name: alias.name,
            alias_source: alias.source,
        });
        self.mode = InputMode::Confirm;
    }

    fn edit_input(&mut self, c: char) {
        let Some(state) = &mut self.edit_state else {
            return;
        };
        match state.active_field {
            EditField::Name => state.name.push(c),
            EditField::Command => state.command.push(c),
        }
    }

    fn edit_backspace(&mut self) {
        let Some(state) = &mut self.edit_state else {
            return;
        };
        match state.active_field {
            EditField::Name => {
                state.name.pop();
            }
            EditField::Command => {
                state.command.pop();
            }
        }
    }

    fn edit_clear(&mut self) {
        let Some(state) = &mut self.edit_state else {
            return;
        };
        match state.active_field {
            EditField::Name => state.name.clear(),
            EditField::Command => state.command.clear(),
        }
    }

    fn edit_toggle_field(&mut self) {
        let Some(state) = &mut self.edit_state else {
            return;
        };
        state.active_field = match state.active_field {
            EditField::Name => EditField::Command,
            EditField::Command => EditField::Name,
        };
        self.mode = InputMode::Editing {
            field: state.active_field,
        };
    }

    fn save_edit(&mut self) {
        let Some(state) = self.edit_state.take() else {
            return;
        };
        let name = state.name.trim().to_string();
        let command = state.command.trim().to_string();

        if let Err(e) = managed::validate_alias_name(&name) {
            self.set_status(&e.to_string());
            self.edit_state = Some(state);
            return;
        }
        if command.is_empty() {
            self.set_status("Command cannot be empty");
            self.edit_state = Some(state);
            return;
        }

        // Check for collisions when name is new or changed
        let name_changed = state.original_name.as_ref() != Some(&name);
        if name_changed {
            if let Some(c) = collision::check_name_collision(&name, &self.groups) {
                self.pending_action = Some(PendingAction::CollisionOverride {
                    edit_state: EditState {
                        name,
                        command,
                        ..state
                    },
                    collision_desc: c.description(),
                });
                self.mode = InputMode::Confirm;
                return;
            }
        }

        let state = EditState {
            name,
            command,
            ..state
        };
        self.do_write(state);
    }

    fn do_write(&mut self, state: EditState) {
        let name = &state.name;
        let command = &state.command;

        if state.is_new {
            if let Err(e) = managed::write_custom_alias(name, command) {
                self.set_status(&e.to_string());
                return;
            }
            self.apply_new_alias(name, command);
            self.show_result(&format!("Created alias: {name}={command}"));
        } else {
            let source = state.original_source.as_ref().unwrap();
            if matches!(source, AliasSource::ZamCustom) {
                if let Err(e) = managed::write_custom_alias(name, command) {
                    self.set_status(&e.to_string());
                    return;
                }
            } else if let Err(e) = managed::write_override(name, command, source) {
                self.set_status(&e.to_string());
                return;
            }
            let orig_name = state.original_name.as_deref().unwrap();
            self.apply_edit_to_groups(orig_name, name, command);
            self.show_result(&format!("Updated alias: {name}={command}"));
        }

        self.rebuild_visible();
    }

    fn cancel_edit(&mut self) {
        self.edit_state = None;
        self.mode = InputMode::Normal {
            focus: PanelFocus::List,
        };
    }

    fn execute_pending(&mut self) {
        let Some(pending) = self.pending_action.take() else {
            return;
        };

        match pending {
            PendingAction::Delete {
                alias_name,
                alias_source,
            } => {
                let result = if matches!(alias_source, AliasSource::ZamCustom) {
                    managed::delete_custom_alias(&alias_name)
                } else {
                    managed::delete_override(&alias_name, &alias_source)
                };

                if let Err(e) = result {
                    self.set_status(&e.to_string());
                    self.mode = InputMode::Normal {
                        focus: PanelFocus::List,
                    };
                } else {
                    self.remove_alias_from_groups(&alias_name);
                    self.show_result(&format!("Deleted alias: {alias_name}"));
                }

                self.rebuild_visible();
            }
            PendingAction::CollisionOverride { edit_state, .. } => {
                self.do_write(edit_state);
            }
        }
    }

    fn cancel_confirm(&mut self) {
        let pending = self.pending_action.take();
        match pending {
            Some(PendingAction::CollisionOverride { edit_state, .. }) => {
                let field = edit_state.active_field;
                self.edit_state = Some(edit_state);
                self.mode = InputMode::Editing { field };
            }
            _ => {
                self.mode = InputMode::Normal {
                    focus: PanelFocus::List,
                };
            }
        }
    }

    // ── In-memory group mutations ─────────────────────────────────────

    fn apply_new_alias(&mut self, name: &str, command: &str) {
        let alias = Alias {
            name: name.to_string(),
            command: command.to_string(),
            source: AliasSource::ZamCustom,
            description: None,
        };

        // Find existing ZamCustom group or create one
        if let Some(group) = self.groups.iter_mut().find(|g| g.source == AliasSource::ZamCustom) {
            group.aliases.push(alias);
            group.aliases.sort_by(|a, b| a.name.cmp(&b.name));
        } else {
            self.groups.push(AliasGroup {
                source: AliasSource::ZamCustom,
                aliases: vec![alias],
                collapsed: false,
            });
            self.groups
                .sort_by(|a, b| a.source.sort_key().cmp(&b.source.sort_key()));
        }
    }

    fn apply_edit_to_groups(&mut self, old_name: &str, new_name: &str, new_command: &str) {
        for group in &mut self.groups {
            for alias in &mut group.aliases {
                if alias.name == old_name {
                    alias.name = new_name.to_string();
                    alias.command = new_command.to_string();
                    return;
                }
            }
        }
    }

    fn remove_alias_from_groups(&mut self, name: &str) {
        for group in &mut self.groups {
            group.aliases.retain(|a| a.name != name);
        }
        self.groups.retain(|g| !g.aliases.is_empty());
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.status_tick = 30; // ~3 seconds at 100ms poll
    }

    fn show_result(&mut self, msg: &str) {
        self.result_message = Some(msg.to_string());
        self.mode = InputMode::ResultPopup;
    }

    /// Called each event loop iteration to count down status messages.
    pub fn tick(&mut self) {
        if self.status_tick > 0 {
            self.status_tick -= 1;
            if self.status_tick == 0 {
                self.status_message = None;
            }
        }
    }

    // ── Existing methods ──────────────────────────────────────────────

    pub fn selected_alias(&self) -> Option<&Alias> {
        match self.visible_items.get(self.selected_index)? {
            ListItem::AliasEntry { alias, .. } => Some(alias),
            ListItem::GroupHeader { group_index, .. } => {
                self.groups.get(*group_index)?.aliases.first()
            }
        }
    }

    pub fn rebuild_visible(&mut self) {
        self.visible_items.clear();
        let (search_field, query) = if self.search_query.starts_with('=') {
            (SearchField::Command, self.search_query[1..].trim().to_lowercase())
        } else if self.search_query.starts_with('@') {
            (SearchField::Group, self.search_query[1..].trim().to_lowercase())
        } else {
            (SearchField::Name, self.search_query.trim().to_lowercase())
        };
        let filtering = !query.is_empty();

        for (gi, group) in self.groups.iter().enumerate() {
            if filtering
                && matches!(search_field, SearchField::Group)
                && !group.source.display_name().to_lowercase().contains(&query)
            {
                continue;
            }

            let matching: Vec<&Alias> = group
                .aliases
                .iter()
                .filter(|a| {
                    if !filtering {
                        return true;
                    }
                    match search_field {
                        SearchField::Name => a.name.to_lowercase().contains(&query),
                        SearchField::Command => a.command.to_lowercase().contains(&query),
                        SearchField::Group => true,
                    }
                })
                .collect();

            if matching.is_empty() && filtering {
                continue;
            }

            self.visible_items.push(ListItem::GroupHeader {
                group_index: gi,
                display_name: group.source.display_name(),
                alias_count: matching.len(),
                collapsed: group.collapsed,
            });

            if !group.collapsed || filtering {
                for alias in matching {
                    self.visible_items.push(ListItem::AliasEntry {
                        group_index: gi,
                        alias: alias.clone(),
                    });
                }
            }
        }
    }

    pub fn compute_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            self.list_scroll_offset = 0;
            return;
        }
        if self.selected_index < self.list_scroll_offset {
            self.list_scroll_offset = self.selected_index;
        }
        if self.selected_index >= self.list_scroll_offset + visible_height {
            self.list_scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    pub fn total_aliases(&self) -> usize {
        self.groups.iter().map(|g| g.aliases.len()).sum()
    }

    pub fn visible_alias_count(&self) -> usize {
        self.visible_items
            .iter()
            .filter(|item| matches!(item, ListItem::AliasEntry { .. }))
            .count()
    }

    pub fn search_field(&self) -> &'static str {
        if self.search_query.starts_with('=') {
            "Command"
        } else if self.search_query.starts_with('@') {
            "Group"
        } else {
            "Search"
        }
    }

    pub fn search_display(&self) -> &str {
        &self.search_query
    }

    fn move_selection(&mut self, delta: isize) {
        if self.visible_items.is_empty() {
            return;
        }
        let max = self.visible_items.len() - 1;
        if delta < 0 {
            self.selected_index = self.selected_index.saturating_sub(delta.unsigned_abs());
        } else {
            self.selected_index = (self.selected_index + delta.unsigned_abs()).min(max);
        }
    }

    fn clamp_selection(&mut self) {
        if self.visible_items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.visible_items.len() {
            self.selected_index = self.visible_items.len() - 1;
        }
    }

    fn toggle_selected_group(&mut self) {
        let group_index = match self.visible_items.get(self.selected_index) {
            Some(
                ListItem::GroupHeader { group_index, .. }
                | ListItem::AliasEntry { group_index, .. },
            ) => *group_index,
            None => return,
        };
        if let Some(group) = self.groups.get_mut(group_index) {
            group.collapsed = !group.collapsed;
        }
        self.rebuild_visible();
    }
}
