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
        }
    }
}

fn print_help() {
    println!(
r#"directree v0.1.0 — AI-optimized project tree

USAGE:
    directree [PATH] [OPTIONS]

ARGS:
    [PATH]    Root directory to scan (default: .)

OPTIONS:
    --depth <N>         Max recursion depth (0 = unlimited)
    --ai                Wrap in <project_structure> XML with annotations — best for AI context
    --flat              Flat list of relative paths — most token-efficient
    --group             Group files by type: source, config, test, assets
    --stats             Show stats: file counts, types, estimated tokens
    --only <exts>       Filter to extensions, comma-separated (e.g. ts,tsx,rs)
    --focus <path>      Tree a subpath with project breadcrumb header
    --collapse          Collapse repeated sibling dir patterns
    --no-color          Disable color output
    --hidden            Show hidden files (dotfiles)
    --show-ignore       Print all active ignore rules and exit
    --no-ignore         Disable built-in ignore rules
    --ignore <pats>     Extra ignore patterns, comma-separated
    --format <fmt>      Output format: tree | flat | json | group
    -h, --help          Show this help
    -V, --version       Show version

EXAMPLES:
    directree                        # tree of current dir
    directree --ai                   # AI context-ready output
    directree --ai --stats           # AI output + token estimate
    directree src --depth 3          # src/ up to depth 3
    directree --flat                 # flat path list (cheapest tokens)
    directree --group                # files grouped by type
    directree --only ts,tsx          # only TypeScript files
    directree --focus src/components # zoom into a subpath
    directree --format json          # machine-readable JSON
    directree --show-ignore          # see what's being filtered
"#
    );
}

pub fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args::default();
    let mut i = 0;

    while i < raw.len() {
        match raw[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("directree 0.1.0");
                std::process::exit(0);
            }
            "--ai"          => args.ai = true,
            "--flat"        => args.flat = true,
            "--group"       => args.group = true,
            "--stats"       => args.stats = true,
            "--collapse"    => args.collapse = true,
            "--no-color"    => args.no_color = true,
            "--hidden"      => args.hidden = true,
            "--show-ignore" => args.show_ignore = true,
            "--no-ignore"   => args.no_ignore = true,
            "--depth" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    args.depth = v.parse().unwrap_or(0);
                }
            }
            "--only" => {
                i += 1;
                args.only = raw.get(i).cloned();
            }
            "--focus" => {
                i += 1;
                args.focus = raw.get(i).cloned();
            }
            "--ignore" => {
                i += 1;
                args.ignore = raw.get(i).cloned();
            }
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
            other if !other.starts_with('-') => {
                args.path = other.to_string();
            }
            other => {
                // handle --depth=3 style
                if let Some(rest) = other.strip_prefix("--depth=") {
                    args.depth = rest.parse().unwrap_or(0);
                } else if let Some(rest) = other.strip_prefix("--only=") {
                    args.only = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--focus=") {
                    args.focus = Some(rest.to_string());
                } else if let Some(rest) = other.strip_prefix("--ignore=") {
                    args.ignore = Some(rest.to_string());
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
