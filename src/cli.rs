/// Parsed CLI arguments
#[derive(Debug, Clone)]
pub struct Args {
    pub path: String,
    pub depth: usize,
    pub ai: bool,
    pub flat: bool,
    pub group: bool,
    pub stats: bool,
    pub only: Option<String>,
    pub focus: Option<String>,
    pub collapse: bool,
    pub no_color: bool,
    pub hidden: bool,
    pub show_ignore: bool,
    pub no_ignore: bool,
    pub ignore: Option<String>,
    pub format: OutputFormat,
    // v0.2 flags
    pub copy: bool,
    pub since: Option<String>,
    pub important: bool,
    pub watch: bool,
    pub init: bool,
    pub completions: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Tree,
    Flat,
    Json,
    Group,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: ".".into(),
            depth: 0,
            ai: false,
            flat: false,
            group: false,
            stats: false,
            only: None,
            focus: None,
            collapse: false,
            no_color: false,
            hidden: false,
            show_ignore: false,
            no_ignore: false,
            ignore: None,
            format: OutputFormat::Tree,
            copy: false,
            since: None,
            important: false,
            watch: false,
            init: false,
            completions: None,
        }
    }
}

fn print_help() {
    println!(
r#"directree v0.2.0 — AI-optimized project tree

USAGE:
    directree [PATH] [OPTIONS]
    directree init              Scaffold .directreeignore for detected framework
    directree completions SHELL Generate shell completions (bash | zsh | fish)

ARGS:
    [PATH]    Root directory to scan (default: .)

OUTPUT MODES:
    --ai                Wrap in <project_structure> XML with annotations
    --flat              Flat list of relative paths (cheapest tokens)
    --group             Group files by type: source, config, test, assets
    --format <fmt>      tree | flat | json | group

FILTERING:
    --depth <N>         Max recursion depth (0 = unlimited)
    --only <exts>       Filter to extensions, comma-separated (e.g. ts,tsx,rs)
    --focus <path>      Tree a subpath with project breadcrumb header
    --since <ref>       Show only files changed since git commit/branch/tag
    --ignore <pats>     Extra ignore patterns, comma-separated
    --no-ignore         Disable built-in ignore rules
    --hidden            Show hidden files (dotfiles)

DISPLAY:
    --stats             File counts, types, estimated token cost
    --important         Surface important files (high imports, recently changed)
    --collapse          Collapse repeated sibling dir patterns
    --no-color          Disable color output

ACTIONS:
    --copy              Copy output to clipboard (pbcopy / xclip / wl-copy)
    --watch             Re-output on filesystem changes
    --show-ignore       Print all active ignore rules and exit

    -h, --help          Show this help
    -V, --version       Show version

EXAMPLES:
    directree --ai --copy         # AI output → straight to clipboard
    directree --ai --stats        # AI output + token estimate
    directree --since HEAD~1      # only files changed in last commit
    directree --important         # highlight high-signal files
    directree --flat --only ts    # ultra-compact TypeScript manifest
    directree --watch --ai        # live-refresh AI context on save
    directree init                # scaffold .directreeignore
    directree completions zsh     # print zsh completions
"#
    );
}

pub fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args::default();

    // Subcommands
    if raw.first().map(|s| s.as_str()) == Some("init") {
        args.init = true;
        return args;
    }
    if raw.first().map(|s| s.as_str()) == Some("completions") {
        args.completions = raw.get(1).cloned().or(Some("bash".into()));
        return args;
    }

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "-h" | "--help" => { print_help(); std::process::exit(0); }
            "-V" | "--version" => { println!("directree 0.2.0"); std::process::exit(0); }
            "--ai"          => args.ai = true,
            "--flat"        => args.flat = true,
            "--group"       => args.group = true,
            "--stats"       => args.stats = true,
            "--collapse"    => args.collapse = true,
            "--no-color"    => args.no_color = true,
            "--hidden"      => args.hidden = true,
            "--show-ignore" => args.show_ignore = true,
            "--no-ignore"   => args.no_ignore = true,
            "--copy"        => args.copy = true,
            "--important"   => args.important = true,
            "--watch"       => args.watch = true,
            "--depth" => {
                i += 1;
                if let Some(v) = raw.get(i) { args.depth = v.parse().unwrap_or(0); }
            }
            "--only" => { i += 1; args.only = raw.get(i).cloned(); }
            "--focus" => { i += 1; args.focus = raw.get(i).cloned(); }
            "--ignore" => { i += 1; args.ignore = raw.get(i).cloned(); }
            "--since" => { i += 1; args.since = raw.get(i).cloned(); }
            "--format" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    args.format = match v.as_str() {
                        "flat"  => OutputFormat::Flat,
                        "json"  => OutputFormat::Json,
                        "group" => OutputFormat::Group,
                        _       => OutputFormat::Tree,
                    };
                }
            }
            other if !other.starts_with('-') => { args.path = other.to_string(); }
            other => {
                // --flag=value style
                if let Some(rest) = other.strip_prefix("--depth=") {
                    args.depth = rest.parse().unwrap_or(0);
                } else if let Some(rest) = other.strip_prefix("--only=") {
                    args.only = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--focus=") {
                    args.focus = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--ignore=") {
                    args.ignore = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--since=") {
                    args.since = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--format=") {
                    args.format = match rest {
                        "flat"  => OutputFormat::Flat,
                        "json"  => OutputFormat::Json,
                        "group" => OutputFormat::Group,
                        _       => OutputFormat::Tree,
                    };
                } else {
                    eprintln!("directree: unknown option '{}'. Use --help for usage.", other);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }
    args
}
