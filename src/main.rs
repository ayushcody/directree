mod cli;
mod clipboard;
mod config;
mod detector;
mod git;
mod importance;
mod init;
mod ignorer;
mod render;
mod tree;
mod walker;
mod watch;

use std::path::PathBuf;
use cli::{OutputFormat, parse_args};
use config::GlobalConfig;
use detector::detect_framework;
use ignorer::Ignorer;
use render::{
    render_tree, render_tree_important, render_ai, render_flat,
    render_group, render_json, render_stats, render_since_header,
};
use walker::Walker;

fn main() {
    let mut args = parse_args();

    // ── Subcommands ────────────────────────────────────────────────────────
    if args.init {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        init::run_init(&root);
        return;
    }
    if let Some(ref shell) = args.completions.clone() {
        init::print_completions(shell);
        return;
    }

    // ── Config merge ───────────────────────────────────────────────────────
    let global = GlobalConfig::load();
    if args.depth == 0 {
        if let Some(d) = global.default_depth { args.depth = d; }
    }

    // ── Resolve root ───────────────────────────────────────────────────────
    let root = PathBuf::from(&args.path);
    let root = match root.canonicalize() {
        Ok(p) => p,
        Err(e) => { eprintln!("directree: cannot access '{}': {}", args.path, e); std::process::exit(1); }
    };
    if !root.is_dir() {
        eprintln!("directree: '{}' is not a directory", args.path);
        std::process::exit(1);
    }

    let scan_root = if let Some(ref focus) = args.focus {
        let p = root.join(focus);
        if !p.exists() { eprintln!("directree: focus '{}' not found", focus); std::process::exit(1); }
        p
    } else {
        root.clone()
    };

    // ── Framework detection ────────────────────────────────────────────────
    let framework = detect_framework(&root);

    // ── Ignore setup ───────────────────────────────────────────────────────
    let mut extra_ignores = global.extra_ignores();
    if let Some(ref cli_ignore) = args.ignore {
        for p in cli_ignore.split(',') {
            let p = p.trim().to_string();
            if !p.is_empty() { extra_ignores.push(p); }
        }
    }
    let mut ignorer = Ignorer::build(&framework, &extra_ignores, args.no_ignore, args.hidden);
    if args.show_ignore {
        println!("directree ignore rules ({} total):\n", ignorer.rules.len());
        for r in &ignorer.rules { println!("  {}", r); }
        return;
    }

    // ── Git --since ────────────────────────────────────────────────────────
    let (since_files, repo_root, since_commits) = if let Some(ref git_ref) = args.since.clone() {
        let repo = git::repo_root(&root);
        let files = git::changed_since(repo.as_deref().unwrap_or(&root), git_ref);
        let commits = repo.as_deref()
            .map(|r| git::recent_commits(r, 5))
            .unwrap_or_default();
        (files, repo, commits)
    } else {
        (None, None, vec![])
    };

    // ── Build tree (inner fn for --watch loop) ─────────────────────────────
    let run_once = |args: &cli::Args| -> String {
        let mut walker = Walker::new(
            args,
            scan_root.clone(),
            framework.clone(),
            Ignorer::build(&framework, &extra_ignores, args.no_ignore, args.hidden),
            since_files.clone(),
            repo_root.clone(),
        );
        let tree = walker.walk();

        // Focus breadcrumb
        let mut out = String::new();
        if args.focus.is_some() {
            let rel = scan_root.strip_prefix(&root)
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            out.push_str(&format!("# focus: ./{}\n", rel));
            out.push_str(&format!("# project root: {}/\n\n",
                root.file_name().and_then(|n| n.to_str()).unwrap_or(".")));
        }

        // --since header
        if let Some(ref git_ref) = args.since {
            let col = !args.no_color;
            out.push_str(&render_since_header(git_ref, &since_commits, col));
        }

        // Importance scores
        let scores = if args.important {
            let flat = tree.flat_paths(&scan_root);
            importance::score_files(&root, &flat)
        } else {
            std::collections::HashMap::new()
        };

        // Render
        let body = if args.ai {
            render_ai(&tree, &framework, args)
        } else if args.flat || args.format == OutputFormat::Flat {
            render_flat(&tree, &scan_root, args)
        } else if args.group || args.format == OutputFormat::Group {
            render_group(&tree, &scan_root, args)
        } else if args.format == OutputFormat::Json {
            render_json(&tree)
        } else if args.important {
            render_tree_important(&tree, args, &scores)
        } else {
            render_tree(&tree, args)
        };

        out.push_str(&body);

        if args.stats && args.format != OutputFormat::Json {
            out.push_str(&render_stats(&tree, args));
        }

        out
    };

    // ── --watch loop ───────────────────────────────────────────────────────
    if args.watch {
        let mut watcher = watch::Watcher::new(800);
        watcher.snapshot(&scan_root);

        loop {
            let output = run_once(&args);

            // Clear screen then print
            print!("\x1b[2J\x1b[H");
            print!("{}", output);

            // Watch header
            let col = !args.no_color;
            if col {
                print!("\x1b[2m── watching for changes (Ctrl+C to exit) ──\x1b[0m\n");
            } else {
                print!("── watching for changes (Ctrl+C to exit) ──\n");
            }

            if args.copy {
                match clipboard::copy_to_clipboard(&output) {
                    Ok(_) => { if col { print!("\x1b[2m✓ copied to clipboard\x1b[0m\n"); } }
                    Err(e) => eprintln!("directree: clipboard: {}", e),
                }
            }

            watcher.wait_for_change(&scan_root);
        }
    }

    // ── Single run ─────────────────────────────────────────────────────────
    let output = run_once(&args);
    print!("{}", output);

    if args.copy {
        // Strip ANSI for clipboard
        let plain = strip_ansi(&output);
        match clipboard::copy_to_clipboard(&plain) {
            Ok(_) => {
                let col = !args.no_color;
                if col { eprintln!("\x1b[2m✓ copied to clipboard\x1b[0m"); }
                else   { eprintln!("✓ copied to clipboard"); }
            }
            Err(e) => eprintln!("directree: --copy failed: {}", e),
        }
    }
}

/// Strip ANSI escape codes for clean clipboard content
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Consume until end of escape sequence (letter)
            for ec in chars.by_ref() {
                if ec.is_ascii_alphabetic() { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}
