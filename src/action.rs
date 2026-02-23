// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoToTop,
    GoToBottom,
    ToggleCollapse,
    FocusSearch,
    ExitSearch,
    SwitchFocus,
    SearchInput(char),
    SearchBackspace,
    SearchClear,

    // CRUD
    CreateAlias,
    EditAlias,
    DeleteAlias,

    // Edit mode
    EditInput(char),
    EditBackspace,
    EditClear,
    EditNextField,
    EditPrevField,
    SaveEdit,
    CancelEdit,

    // Confirm mode
    ConfirmYes,
    ConfirmNo,

    // Result popup
    DismissResult,

    // Help
    ShowHelp,
    CloseHelp,
}
