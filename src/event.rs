// SPDX-License-Identifier: Apache-2.0

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::action::Action;
use crate::app::InputMode;

/// Poll for the next action, returning None on timeout or non-key events.
pub fn next_action(mode: InputMode) -> std::io::Result<Option<Action>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => Ok(map_key(key, mode)),
        _ => Ok(None),
    }
}

fn map_key(key: KeyEvent, mode: InputMode) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }

    match mode {
        InputMode::Search => map_search_key(key),
        InputMode::Normal { .. } => map_normal_key(key),
        InputMode::Editing { .. } => map_editing_key(key),
        InputMode::Confirm => map_confirm_key(key),
        InputMode::ResultPopup => Some(Action::DismissResult),
        InputMode::Help => map_help_key(key),
    }
}

fn map_search_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => Some(Action::ExitSearch),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Tab => Some(Action::SwitchFocus),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                Some(Action::SearchClear)
            } else {
                Some(Action::SearchInput(c))
            }
        }
        _ => None,
    }
}

fn map_normal_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('/') => Some(Action::FocusSearch),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollUp),
        KeyCode::Enter | KeyCode::Char(' ') => Some(Action::ToggleCollapse),
        KeyCode::Tab => Some(Action::SwitchFocus),
        KeyCode::Char('g') => Some(Action::GoToTop),
        KeyCode::Char('G') => Some(Action::GoToBottom),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::PageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::PageUp)
        }
        KeyCode::Char('a') => Some(Action::CreateAlias),
        KeyCode::Char('e') => Some(Action::EditAlias),
        KeyCode::Char('d') => Some(Action::DeleteAlias),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        _ => None,
    }
}

fn map_editing_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::CancelEdit),
        KeyCode::Enter => Some(Action::SaveEdit),
        KeyCode::Tab => Some(Action::EditNextField),
        KeyCode::BackTab => Some(Action::EditPrevField),
        KeyCode::Backspace => Some(Action::EditBackspace),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                Some(Action::EditClear)
            } else {
                Some(Action::EditInput(c))
            }
        }
        _ => None,
    }
}

fn map_confirm_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Some(Action::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::ConfirmNo),
        _ => None,
    }
}

fn map_help_key(_key: KeyEvent) -> Option<Action> {
    Some(Action::CloseHelp)
}
