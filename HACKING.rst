===============
Hacking on zam
===============

Project structure
=================

::

    src/
      main.rs               entry point, arg parsing, terminal setup, ext subcommand
      action.rs              Action enum (Copy, no heap)
      event.rs               crossterm key events -> Action
      app.rs                 AppState, InputMode, selection, search filter, CRUD
      managed.rs             managed directory I/O (~/.config/zam/)
      extensions.rs          static extension definitions (alias packs)
      collision.rs           collision detection (aliases, commands, builtins, functions)
      alias/
        mod.rs               load_aliases() — top-level pipeline entry
        types.rs             AliasSource, Alias, AliasGroup, CollectError, ManagedError
        collector.rs         spawn zsh, capture alias output
        parser.rs            parse alias lines (quoting edge cases, unit tests)
        categorizer.rs       scan source files, attribute aliases to origins
      ui/
        mod.rs               top-level render() that composes all panels
        layout.rs            top-level rect splitting
        alias_list.rs        left panel: grouped scrollable list
        detail_panel.rs      right panel: alias details
        edit_form.rs         right panel: edit form (create/edit mode)
        confirm_dialog.rs    right panel: delete and collision confirmation
        search_bar.rs        top: search input with mode hints
        help_bar.rs          bottom: context-sensitive keybindings
        help_panel.rs        right panel: help screen
        theme.rs             colors and styles

Data pipeline
=============

::

    collect -> parse -> categorize -> render

Each stage is a separate module with no coupling to the others.

``alias/collector.rs`` spawns zsh (``zsh -ic 'alias'``) and captures
stdout. ``alias/parser.rs`` turns those lines into ``(name, command)``
pairs. ``alias/categorizer.rs`` scans source files to attribute each
alias to its origin, then groups them. The UI modules are pure functions
that take state and produce widgets.

Key types
=========

``Action`` (``action.rs``)
    A Copy enum of every possible user action. No heap allocations.
    ``event.rs`` maps crossterm key events to ``Action`` values based
    on the current ``InputMode``.

``InputMode`` (``app.rs``)
    An enum with five variants::

        enum InputMode {
            Normal { focus: PanelFocus },
            Search,
            Editing { field: EditField },
            Confirm,
            Help,
        }

    This makes it impossible to be in search mode while focusing the
    detail panel. The type system enforces valid states.

``EditState`` (``app.rs``)
    Holds the name, command, active field, is_new flag, and original
    name/source for edits. Populated when entering Editing mode.

``PendingAction`` (``app.rs``)
    An enum with two variants::

        enum PendingAction {
            Delete { alias_name, alias_source },
            CollisionOverride { edit_state, collision_desc },
        }

    ``Delete`` is populated when entering Confirm mode for a deletion.
    ``CollisionOverride`` is populated when ``save_edit()`` detects a
    name collision and the user must decide whether to override.

``SearchField`` (``app.rs``)
    Private enum controlling what the search query matches against::

        enum SearchField {
            Name,
            Command,
            Group,
        }

    The first character of the query selects the field: ``=`` for
    command, ``@`` for group, anything else for alias name.

``AliasSource`` (``alias/types.rs``)
    Enum of every known origin::

        enum AliasSource {
            OhMyZshPlugin(String),
            OhMyZshLib(String),
            OhMyZshCustom(String),
            Zshrc,
            ZamExtension(String),
            ZamCustom,
            ZamOverride(String),
            Unknown,
        }

    Drives grouping, display names, file paths, and sort order.

``Collision`` (``collision.rs``)
    Enum representing what a name collides with::

        enum Collision {
            Alias { source },
            Command,
            Builtin,
            Function,
        }

``ManagedError`` (``alias/types.rs``)
    Error type for managed directory operations (create dir, write
    file, not initialized, invalid name).

``Extension`` (``extensions.rs``)
    Static extension definition: name, description, and a slice of
    ``(alias_name, command, comment)`` tuples. Four extensions are
    compiled into the binary: sysadmin, rust, c, python.

Managed directory
=================

``zam --init`` creates::

    ~/.config/zam/
      init.zsh           sources aliases + overrides, defines zam() wrapper
      aliases/
        custom.zsh       user-created aliases
        {ext}.zsh        installed extension packs
      overrides/
        {slug}.zsh       per-source overrides and deletions

``managed.rs`` handles all disk I/O: creating directories, writing
alias lines, removing alias lines, writing ``unalias`` lines for
deletions, and installing/removing extension files.

``zam --nuke`` deletes ``~/.config/zam/`` entirely and removes the
source line from ``~/.zshrc``.

The ``init.zsh`` file includes a ``zam()`` shell function that wraps
the binary. After ``zam ext install`` or ``zam ext remove`` succeeds,
the wrapper re-sources ``init.zsh`` so new aliases are available
immediately.

CRUD operations
===============

**Create (a):** Enter edit mode with empty fields. On save, check for
name collisions. If collision found, show override dialog. Otherwise
write to ``custom.zsh``. Add to in-memory ZamCustom group.

**Edit (e):** Enter edit mode pre-filled from selected alias. On save:
if name changed, check for collisions. If source is ZamCustom, update
``custom.zsh``; otherwise write override.

**Delete (d):** Show confirmation. On confirm: if source is ZamCustom,
remove line from ``custom.zsh``; otherwise add ``unalias`` to override
file. Remove from in-memory groups.

All three check ``is_initialized()`` first.

Collision detection
===================

``collision.rs`` checks for name collisions against two sources:

1. **In-memory aliases** — iterates all groups looking for a matching name
2. **Shell type check** — runs ``zsh -c 'whence -w name'`` and parses
   the output to detect commands, builtins, and functions

In the TUI, collision detection runs during ``save_edit()`` when the
alias name is new or changed. If a collision is found, a
``PendingAction::CollisionOverride`` is created and the UI enters
Confirm mode. Pressing ``y`` writes the alias anyway; pressing ``n``
or ``Esc`` returns to the edit form with the state preserved.

For ``zam ext install``, ``check_batch_collisions()`` runs all alias
names in the extension against the current alias set and reports
conflicts before prompting to proceed.

Extensions
==========

``extensions.rs`` defines four alias packs as static data compiled
into the binary:

- **sysadmin** — disk, network, process, file ops (20 aliases)
- **rust** — cargo shortcuts (15 aliases)
- **c** — gcc, cmake, make, valgrind (10 aliases)
- **python** — python3, pip, venv, pytest (13 aliases)

Public API: ``EXTENSIONS`` (const slice), ``find_extension(name)``.

Extension files are written to ``~/.config/zam/aliases/{name}.zsh``
with comments above each alias. The categorizer scans all ``.zsh``
files in the aliases directory (excluding ``custom.zsh``) and
attributes them as ``ZamExtension(name)``.

Adding a new extension
----------------------

1. Add an ``Extension`` entry to the ``EXTENSIONS`` slice in
   ``extensions.rs`` with name, description, and alias tuples.
2. That's it. The install/remove commands, categorizer, and TUI
   grouping all work automatically.

Search
======

Search uses case-insensitive substring matching. No fuzzy matching.

========== ========================= ==============
Prefix     Searches                  Label
========== ========================= ==============
*(none)*   alias names               ``Search:``
``=``      alias commands            ``Command:``
``@``      group display names       ``Group:``
========== ========================= ==============

When the search bar is empty, placeholder text shows the available
modes: ``name  =command  @group``.

Dependencies
============

Four crates, all well-maintained, no optional features enabled beyond
defaults:

- **ratatui 0.30** — TUI framework
- **crossterm 0.29** — terminal backend
- **dirs 6** — home directory resolution (cross-platform)
- **thiserror 2** — error type derivation

No async runtime. No serde. No clap. The arg parsing is a few lines of
``match`` on string slices because that is all it needs to be.

Testing
=======

::

    cargo test

The parser has unit tests covering zsh format, double-quoted values,
``$'...'`` ANSI-C quoting, the ``'\''`` single-quote idiom, embedded
equals signs, and multi-line input.

Development mockup
==================

::

    zam --mockup

Renders a single frame of the TUI to stdout at a fixed size using
ratatui's ``TestBackend``. Useful for iterating on the README mockup
without launching the full interactive TUI. This flag is not shown in
``--help``.

Adding a new search mode
========================

1. Add a variant to ``SearchField`` in ``app.rs``.
2. Pick a prefix character and add a branch in ``rebuild_visible()``
   where the ``(search_field, query)`` tuple is constructed.
3. Add the matching logic inside the filter closure (or before it, as
   ``@group`` does for group-level filtering).
4. Update ``search_field()`` to return the label string.
5. Update the placeholder text in ``ui/search_bar.rs``.
6. Add a hint in ``ui/help_bar.rs``.
