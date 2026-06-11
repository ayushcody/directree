use std::path::Path;
use crate::detector::{Framework, detect_framework};

/// Scaffold a .directreeignore in the current directory
pub fn run_init(root: &Path) {
    let framework = detect_framework(root);
    let content = generate_ignore_file(&framework);
    let target = root.join(".directreeignore");

    if target.exists() {
        eprintln!("directree: .directreeignore already exists — not overwriting.");
        eprintln!("           Delete it first or edit it manually.");
        std::process::exit(1);
    }

    match std::fs::write(&target, &content) {
        Ok(_) => {
            println!("✓ Created .directreeignore (detected framework: {})", framework.name());
            println!();
            println!("{}", content);
        }
        Err(e) => {
            eprintln!("directree: failed to write .directreeignore: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_ignore_file(framework: &Framework) -> String {
    let mut lines: Vec<&str> = Vec::new();

    lines.push("# .directreeignore — directree-specific ignores");
    lines.push("# Syntax: same as .gitignore");
    lines.push("# These are ADDED on top of directree's built-in ignore rules.");
    lines.push("# Run `directree --show-ignore` to see everything being filtered.");
    lines.push("");

    lines.push("# ── Generated files ──────────────────────────────");

    match framework {
        Framework::NextJs | Framework::React | Framework::Vite => {
            lines.push("**/*.generated.ts");
            lines.push("**/*.generated.js");
            lines.push("src/graphql/generated/");
            lines.push("next-env.d.ts");
        }
        Framework::Rust => {
            lines.push("**/*.generated.rs");
            lines.push("build.rs.out/");
        }
        Framework::Python => {
            lines.push("**/*_pb2.py");
            lines.push("**/*_pb2_grpc.py");
            lines.push("**/migrations/");
        }
        Framework::Go => {
            lines.push("**/*.pb.go");
            lines.push("**/*_gen.go");
        }
        _ => {
            lines.push("**/*.generated.*");
        }
    }

    lines.push("");
    lines.push("# ── Secrets / environment ────────────────────────");
    lines.push(".env.local");
    lines.push(".env.production");
    lines.push(".env.staging");
    lines.push("*.pem");
    lines.push("*.key");
    lines.push("secrets/");

    lines.push("");
    lines.push("# ── Large / binary assets ────────────────────────");
    lines.push("public/videos/");
    lines.push("public/fonts/");
    lines.push("assets/raw/");
    lines.push("*.mp4");
    lines.push("*.mov");

    lines.push("");
    lines.push("# ── Logs & temp ──────────────────────────────────");
    lines.push("logs/");
    lines.push("*.log");
    lines.push("*.tmp");

    match framework {
        Framework::NextJs => {
            lines.push("");
            lines.push("# ── Next.js specific ────────────────────────────");
            lines.push("# Uncomment to hide storybook output:");
            lines.push("# storybook-static/");
            lines.push("# Uncomment to hide Prisma migrations:");
            lines.push("# prisma/migrations/");
        }
        Framework::Rust => {
            lines.push("");
            lines.push("# ── Rust specific ───────────────────────────────");
            lines.push("benches/");
            lines.push("examples/");
            lines.push("# Uncomment to hide docs:");
            lines.push("# docs/");
        }
        Framework::Python => {
            lines.push("");
            lines.push("# ── Python specific ─────────────────────────────");
            lines.push("*.pyc");
            lines.push("*.pyo");
            lines.push("__pycache__/");
            lines.push("*.egg-info/");
            lines.push("htmlcov/");
        }
        _ => {}
    }

    lines.push("");
    lines.push("# ── Add your custom patterns below ──────────────");
    lines.push("");

    lines.join("\n") + "\n"
}


// ── Shell completions ─────────────────────────────────────────────────────────

pub fn print_completions(shell: &str) {
    match shell.to_lowercase().as_str() {
        "bash" => print!("{}", BASH_COMPLETIONS),
        "zsh"  => print!("{}", ZSH_COMPLETIONS),
        "fish" => print!("{}", FISH_COMPLETIONS),
        other  => {
            eprintln!("directree: unknown shell '{}'. Supported: bash, zsh, fish", other);
            std::process::exit(1);
        }
    }
}

const BASH_COMPLETIONS: &str = r#"# directree bash completions
# Add to ~/.bashrc:  source <(directree completions bash)

_directree_complete() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="--ai --flat --group --stats --collapse --no-color --hidden
          --show-ignore --no-ignore --copy --important --watch
          --depth --only --focus --since --ignore --format
          -h --help -V --version
          init completions"

    case "$prev" in
        --format)
            COMPREPLY=( $(compgen -W "tree flat json group" -- "$cur") )
            return 0 ;;
        --depth)
            COMPREPLY=( $(compgen -W "1 2 3 4 5 6" -- "$cur") )
            return 0 ;;
        --since)
            COMPREPLY=( $(compgen -W "HEAD HEAD~1 HEAD~2 main master develop" -- "$cur") )
            return 0 ;;
        completions)
            COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") )
            return 0 ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
    else
        COMPREPLY=( $(compgen -d -- "$cur") )
    fi
}

complete -F _directree_complete directree
complete -F _directree_complete dt
"#;

const ZSH_COMPLETIONS: &str = r#"#compdef directree dt
# directree zsh completions
# Add to ~/.zshrc:  source <(directree completions zsh)

_directree() {
    local -a opts dirs

    _arguments \
        '(-h --help)'{-h,--help}'[Show help]' \
        '(-V --version)'{-V,--version}'[Show version]' \
        '--ai[AI context-ready XML output]' \
        '--flat[Flat path list]' \
        '--group[Group files by type]' \
        '--stats[Show stats and token estimate]' \
        '--collapse[Collapse repeated sibling patterns]' \
        '--copy[Copy output to clipboard]' \
        '--important[Surface high-signal files]' \
        '--watch[Re-output on filesystem changes]' \
        '--no-color[Disable color]' \
        '--hidden[Show dotfiles]' \
        '--show-ignore[Print active ignore rules]' \
        '--no-ignore[Disable built-in ignore rules]' \
        '--depth[Max depth]:depth:(1 2 3 4 5 6 0)' \
        '--only[Extension filter]:extensions:' \
        '--focus[Subpath focus]:path:_files -/' \
        '--since[Changed since git ref]:git-ref:(HEAD HEAD~1 HEAD~2 main master develop)' \
        '--ignore[Extra ignore patterns]:patterns:' \
        '--format[Output format]:format:(tree flat json group)' \
        '1:directory:_files -/' \
        ':subcommands:(init completions)'
}

_directree
"#;

const FISH_COMPLETIONS: &str = r#"# directree fish completions
# Save to ~/.config/fish/completions/directree.fish

complete -c directree -f
complete -c directree -s h -l help       -d 'Show help'
complete -c directree -s V -l version    -d 'Show version'
complete -c directree -l ai              -d 'AI context-ready XML output with annotations'
complete -c directree -l flat            -d 'Flat list of relative paths'
complete -c directree -l group           -d 'Group files by type'
complete -c directree -l stats           -d 'Show stats and token estimate'
complete -c directree -l collapse        -d 'Collapse repeated sibling patterns'
complete -c directree -l copy            -d 'Copy output to clipboard'
complete -c directree -l important       -d 'Surface high-signal files'
complete -c directree -l watch           -d 'Re-output on filesystem changes'
complete -c directree -l no-color        -d 'Disable color output'
complete -c directree -l hidden          -d 'Show dotfiles'
complete -c directree -l show-ignore     -d 'Print active ignore rules'
complete -c directree -l no-ignore       -d 'Disable built-in ignore rules'
complete -c directree -l depth           -d 'Max recursion depth' -r
complete -c directree -l only            -d 'Filter extensions (e.g. ts,tsx)' -r
complete -c directree -l focus           -d 'Zoom into subpath' -r -a '(__fish_complete_directories)'
complete -c directree -l since           -d 'Changed since git ref' -r -a 'HEAD HEAD~1 HEAD~2 main master develop'
complete -c directree -l ignore          -d 'Extra ignore patterns' -r
complete -c directree -l format          -d 'Output format' -r -a 'tree flat json group'

# Subcommands
complete -c directree -n '__fish_use_subcommand' -a init        -d 'Scaffold .directreeignore'
complete -c directree -n '__fish_use_subcommand' -a completions -d 'Print shell completions'
complete -c directree -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish'

# Directory completion
complete -c directree -a '(__fish_complete_directories)'
"#;
