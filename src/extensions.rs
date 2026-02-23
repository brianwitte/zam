// SPDX-License-Identifier: Apache-2.0

/// A curated alias pack shipped with the binary.
pub struct Extension {
    pub name: &'static str,
    pub description: &'static str,
    pub aliases: &'static [(&'static str, &'static str, &'static str)],
}

pub static EXTENSIONS: &[Extension] = &[
    Extension {
        name: "sysadmin",
        description: "Disk, network, process, and file operations",
        aliases: &[
            ("dfh", "df -h", "Disk free (human-readable)"),
            ("duh", "du -h -d1", "Disk usage depth 1"),
            ("dus", "du -sh", "Disk usage summary"),
            ("mnt", "mount | column -t", "Show mounts in columns"),
            ("ports", "lsof -i -P -n | grep LISTEN", "Listening ports"),
            ("myip", "curl -s ifconfig.me", "Public IP address"),
            ("pingg", "ping -c 5 8.8.8.8", "Ping Google DNS"),
            ("httph", "curl -sI", "HTTP headers only"),
            ("psg", "ps aux | grep -i", "Search processes"),
            ("psmem", "ps aux -m | head -20", "Top processes by memory"),
            ("pscpu", "ps aux -r | head -20", "Top processes by CPU"),
            ("k9", "kill -9", "Force kill"),
            ("lsd", "ls -ld */", "List directories only"),
            ("lsz", "ls -lhS", "List files by size"),
            ("cpv", "cp -iv", "Copy verbose + confirm"),
            ("mvv", "mv -iv", "Move verbose + confirm"),
            ("mkp", "mkdir -p", "Make directory (parents)"),
            ("rmrf", "rm -rf", "Remove recursively (force)"),
            ("reload", "exec zsh", "Reload shell"),
            ("path", "echo $PATH | tr : \\\\n", "Show PATH entries"),
        ],
    },
    Extension {
        name: "rust",
        description: "Cargo shortcuts for Rust development",
        aliases: &[
            ("cb", "cargo build", "Build"),
            ("cr", "cargo run", "Run"),
            ("ct", "cargo test", "Test"),
            ("cck", "cargo check", "Check"),
            ("ccl", "cargo clippy", "Lint"),
            ("cfmt", "cargo fmt", "Format"),
            ("cdoc", "cargo doc --open", "Build and open docs"),
            ("crel", "cargo build --release", "Release build"),
            ("crr", "cargo run --release", "Release run"),
            ("cadd", "cargo add", "Add dependency"),
            ("crm", "cargo remove", "Remove dependency"),
            ("cup", "cargo update", "Update dependencies"),
            ("cfix", "cargo fix --allow-dirty", "Auto-fix lints"),
            ("cwat", "cargo watch -x test", "Watch and test"),
            ("cben", "cargo bench", "Benchmarks"),
        ],
    },
    Extension {
        name: "c",
        description: "C programming and build tools",
        aliases: &[
            ("cc99", "gcc -std=c99 -Wall -Wextra -pedantic", "C99 strict"),
            ("cc11", "gcc -std=c11 -Wall -Wextra -pedantic", "C11 strict"),
            ("cdbg", "gcc -g -O0 -Wall -Wextra -DDEBUG", "Debug build"),
            ("copt", "gcc -O2 -Wall -Wextra", "Optimized build"),
            ("casm", "gcc -S -masm=intel", "Intel assembly output"),
            (
                "cmk",
                "cmake -B build && cmake --build build",
                "CMake configure + build",
            ),
            (
                "cmkr",
                "cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build",
                "CMake release build",
            ),
            ("mkj", "make -j$(sysctl -n hw.ncpu)", "Make (all cores)"),
            ("mkc", "make clean", "Make clean"),
            ("vg", "valgrind --leak-check=full", "Valgrind leak check"),
        ],
    },
    Extension {
        name: "python",
        description: "Python development and virtual environments",
        aliases: &[
            ("py", "python3", "Python 3"),
            ("ipy", "ipython", "IPython REPL"),
            ("pyserv", "python3 -m http.server", "HTTP server"),
            ("pyjson", "python3 -m json.tool", "Pretty-print JSON"),
            (
                "pyprof",
                "python3 -m cProfile -s cumtime",
                "Profile script",
            ),
            ("venv", "python3 -m venv .venv", "Create virtualenv"),
            ("va", "source .venv/bin/activate", "Activate virtualenv"),
            ("pipi", "pip install", "Pip install"),
            (
                "pipir",
                "pip install -r requirements.txt",
                "Install requirements",
            ),
            (
                "pipf",
                "pip freeze > requirements.txt",
                "Freeze requirements",
            ),
            ("pipu", "pip install --upgrade", "Pip upgrade"),
            ("pyt", "python3 -m pytest", "Run pytest"),
            ("pytv", "python3 -m pytest -v", "Run pytest verbose"),
        ],
    },
];

pub fn find_extension(name: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|e| e.name == name)
}
