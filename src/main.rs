// SPDX-License-Identifier: Apache-2.0

mod action;
mod alias;
mod app;
mod collision;
mod event;
mod extensions;
mod managed;
mod ui;

use std::env;
use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use alias::types::AliasGroup;
use app::AppState;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    if args.iter().any(|a| a == "--init") {
        return run_init();
    }

    if args.iter().any(|a| a == "--nuke") {
        return run_nuke();
    }

    // ext subcommand: zam ext [list|install|remove] [name]
    if args.get(1).map(|s| s.as_str()) == Some("ext") {
        return run_ext(&args[2..]);
    }

    let list_mode = args.iter().any(|a| a == "--list" || a == "-l");
    let mockup_mode = args.iter().any(|a| a == "--mockup");

    eprintln!("Loading aliases from zsh...");
    let groups = match alias::load_aliases() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    if list_mode {
        print_list(&groups);
        return Ok(());
    }

    if mockup_mode {
        print_mockup(groups);
        return Ok(());
    }

    run_tui(groups)
}

fn run_init() -> io::Result<()> {
    match managed::init() {
        Ok(()) => {
            println!("Initialized ~/.config/zam/");
            println!();
            println!("Restart your shell or run:");
            println!("  source ~/.config/zam/init.zsh");
            Ok(())
        }
        Err(e) => {
            eprintln!("Init failed: {e}");
            std::process::exit(1);
        }
    }
}

fn run_nuke() -> io::Result<()> {
    eprint!("This will delete ~/.config/zam/ and remove the source line from ~/.zshrc. Continue? [y/N] ");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if !answer.trim().eq_ignore_ascii_case("y") {
        println!("Aborted.");
        return Ok(());
    }

    match managed::nuke() {
        Ok(()) => {
            println!("Removed ~/.config/zam/ and cleaned ~/.zshrc");
            println!();
            println!("Restart your shell or run:");
            println!("  unfunction zam 2>/dev/null; source ~/.zshrc");
        }
        Err(e) => {
            eprintln!("Nuke failed: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn run_ext(args: &[String]) -> io::Result<()> {
    let subcmd = args.first().map(|s| s.as_str()).unwrap_or("list");

    match subcmd {
        "list" | "" => ext_list(),
        "install" => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
            ext_install(name)
        }
        "remove" => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
            ext_remove(name)
        }
        other => {
            eprintln!("Unknown ext command: {other}");
            eprintln!("Usage: zam ext [list|install|remove] [name]");
            std::process::exit(1);
        }
    }
}

fn ext_list() -> io::Result<()> {
    println!("Available extensions:\n");
    for ext in extensions::EXTENSIONS {
        let status = if managed::is_extension_installed(ext.name) {
            "installed"
        } else {
            "not installed"
        };
        println!("  {:<12} {} [{}]", ext.name, ext.description, status);
    }
    println!();
    println!("Install with: zam ext install <name>");
    println!("Remove with:  zam ext remove <name>");
    Ok(())
}

fn ext_install(name: &str) -> io::Result<()> {
    if name.is_empty() {
        eprintln!("Usage: zam ext install <name>");
        eprintln!("Run `zam ext` to see available extensions.");
        std::process::exit(1);
    }

    let Some(ext) = extensions::find_extension(name) else {
        eprintln!("Unknown extension: {name}");
        eprintln!("Run `zam ext` to see available extensions.");
        std::process::exit(1);
    };

    if managed::is_extension_installed(name) {
        println!("Extension '{name}' is already installed.");
        return Ok(());
    }

    if !managed::is_initialized() {
        eprintln!("Not initialized — run `zam --init` first.");
        std::process::exit(1);
    }

    // Load current aliases for collision checking
    let groups = alias::load_aliases().unwrap_or_default();

    let collisions = collision::check_batch_collisions(ext.aliases, &groups);
    if !collisions.is_empty() {
        println!("Collisions detected:");
        for (alias_name, c) in &collisions {
            println!("  {alias_name}: {}", c.description());
        }
        println!();
        eprint!("Install anyway? [y/N] ");
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    match managed::install_extension(name, ext.aliases) {
        Ok(()) => {
            println!(
                "Installed extension '{name}' ({} aliases)",
                ext.aliases.len()
            );
        }
        Err(e) => {
            eprintln!("Failed to install: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn ext_remove(name: &str) -> io::Result<()> {
    if name.is_empty() {
        eprintln!("Usage: zam ext remove <name>");
        std::process::exit(1);
    }

    if extensions::find_extension(name).is_none() {
        eprintln!("Unknown extension: {name}");
        std::process::exit(1);
    }

    if !managed::is_extension_installed(name) {
        println!("Extension '{name}' is not installed.");
        return Ok(());
    }

    match managed::remove_extension(name) {
        Ok(()) => {
            println!("Removed extension '{name}'");
        }
        Err(e) => {
            eprintln!("Failed to remove: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn run_tui(groups: Vec<AliasGroup>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(groups);
    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Some(action) = event::next_action(app.mode)? {
            app.handle_action(action);
        }

        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

fn print_list(groups: &[AliasGroup]) {
    let total: usize = groups.iter().map(|g| g.aliases.len()).sum();
    println!("{} aliases in {} groups (zsh)\n", total, groups.len());

    for group in groups {
        let name = group.source.display_name();
        let file = group.source.file_path().unwrap_or_default();

        println!("── {} ({}) ──", name, group.aliases.len());
        if !file.is_empty() {
            println!("   {file}");
        }
        println!();

        let max_name = group
            .aliases
            .iter()
            .map(|a| a.name.len())
            .max()
            .unwrap_or(0);

        for alias in &group.aliases {
            print!(
                "  {:<width$}  {}",
                alias.name,
                alias.command,
                width = max_name
            );
            if let Some(desc) = &alias.description {
                print!("  # {desc}");
            }
            println!();
        }
        println!();
    }
}

fn print_mockup(groups: Vec<AliasGroup>) {
    use ratatui::backend::TestBackend;

    let width = 90;
    let height = 18;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("failed to create test terminal");
    let mut app = AppState::new(groups);

    terminal
        .draw(|frame| ui::render(frame, &mut app))
        .expect("failed to render");

    let buf = terminal.backend().buffer().clone();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buf.cell((x, y)).map_or(" ", |c| c.symbol()));
        }
        println!("{}", line.trim_end());
    }
}

fn print_usage() {
    println!("zam - zsh alias manager\n");
    println!("USAGE:");
    println!("  zam                Launch interactive TUI");
    println!("  zam --list         Print all aliases grouped by source");
    println!("  zam --init         Initialize managed alias directory (~/.config/zam/)");
    println!("  zam --nuke         Delete all zam config and clean ~/.zshrc");
    println!("  zam ext            List available extensions");
    println!("  zam ext install <name>  Install an extension");
    println!("  zam ext remove <name>   Remove an extension");
    println!("\nOPTIONS:");
    println!("  -l, --list         Print aliases to stdout instead of launching TUI");
    println!("  --init             Set up ~/.config/zam/ and add source line to ~/.zshrc");
    println!("  --nuke             Remove ~/.config/zam/ and source line from ~/.zshrc");
    println!("  -h, --help         Show this help message");
}
