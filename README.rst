===
zam
===

:Binary: ``zam``
:Version: 0.1.0
:Shell support: zsh

``zam`` (zsh alias manager) is a TUI for browsing and managing zsh
aliases. It collects every active alias, traces each one back to the
file that defined it, and presents the result as a searchable, grouped,
collapsible list with full CRUD.

::

    ┌ Search ────────────────────────────────────────────────────────────────────────────────┐
    │[/] Search: name  =command  @group  (269 aliases)                                       │
    └────────────────────────────────────────────────────────────────────────────────────────┘
    ┌ Aliases ──────────────────────────────┐┌ Details ──────────────────────────────────────┐
    │ ▾ oh-my-zsh lib: directories.zsh (19) ││                                               │
    │   ...     ../..                       ││  Alias:   ...                                 │
    │   ....    ../../..                    ││                                               │
    │   .....   ../../../..                 ││  Command:                                     │
    │   ......  ../../../../..              ││             ../..                             │
    │   1       cd -1                       ││                                               │
    │   2       cd -2                       ││  Source:   oh-my-zsh lib: directories.zsh     │
    │   3       cd -3                       ││                                               │
    │   4       cd -4                       ││  File:    ~/.oh-my-zsh/lib/directories.zsh    │
    │   5       cd -5                       ││                                               │
    │   6       cd -6                       ││                                               │
    └───────────────────────────────────────┘└───────────────────────────────────────────────┘
    /:search        a:add           e:edit          d:delete        ?:help          q:quit
    j/k:navigate    g/G:top/bottom  ^d/u:page       Enter:toggle    Tab:focus

Building
========

::

    cargo install --path .

Setup
=====

::

    zam --init

Creates ``~/.config/zam/``, writes ``init.zsh``, adds a source line to
``~/.zshrc``. Restart your shell once afterwards. The ``init.zsh``
wrapper auto-reloads aliases after extension install/remove.

::

    zam --nuke

Deletes ``~/.config/zam/`` and removes the source line from ``~/.zshrc``.

Usage
=====

::

    zam              # TUI
    zam --list       # dump aliases to stdout
    zam --init       # initialize ~/.config/zam/
    zam --nuke       # delete all zam config
    zam --help       # usage

Extensions
----------

Opt-in alias packs shipped with the binary::

    zam ext              # list extensions
    zam ext install rust # install
    zam ext remove rust  # remove

============ ==========================================
Name         Description
============ ==========================================
``sysadmin`` Disk, network, process, file ops
``rust``     Cargo shortcuts
``c``        gcc, cmake, make, valgrind
``python``   python3, pip, venv, pytest
============ ==========================================

Collision warnings are shown before overwriting existing aliases,
commands, builtins, or functions.

How it works
============

``zam`` spawns ``zsh -ic 'alias'``, parses the output (handles all zsh
quoting styles), then scans source files in load order to attribute
each alias::

    ~/.oh-my-zsh/lib/*.zsh
    ~/.oh-my-zsh/plugins/{enabled}/*.plugin.zsh
    ~/.oh-my-zsh/custom/*.zsh
    ~/.zshrc
    ~/.config/zam/aliases/*.zsh

First match wins. Unmatched aliases go to "Unknown".

Comments above alias definitions are extracted as descriptions.

``~/.config/zam/`` layout after init::

    init.zsh               sources everything, defines zam() wrapper
    aliases/custom.zsh     user-created aliases
    aliases/{ext}.zsh      installed extensions
    overrides/{slug}.zsh   per-source overrides and unalias deletions

Hacking
=======

See `HACKING.rst <HACKING.rst>`_.

License
=======

Apache-2.0.
