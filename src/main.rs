mod cli;
mod config;
mod detector;
mod ignorer;
mod render;
mod tree;
mod walker;

use std::path::PathBuf;
use cli::{OutputFormat, parse_args};
use config::GlobalConfig;
use detector::detect_framework;
use ignorer::Ignorer;
use render::{render_tree, render_ai, render_flat, render_group, render_json, render_stats};
use walker::Walker;

fn main() {
    let mut args = parse_args();

    let global = GlobalConfig::load();
    if args.depth == 0 {
        if let Some(d) = global.default_depth {
            args.depth = d;
        }
    }

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
        let focused = root.join(focus);
        if !focused.exists() {
            eprintln!("directree: focus path '{}' does not exist", focus);
            std::process::exit(1);
        }
        focused
    } else {
        root.clone()
    };

    let framework = detect_framework(&root);

    let mut extra_ignores = global.extra_ignores();
    if let Some(ref cli_ignore) = args.ignore {
        for pat in cli_ignore.split(',') {
            let pat = pat.trim().to_string();
            if !pat.is_empty() { extra_ignores.push(pat); }
        }
    }

    let mut ignorer = Ignorer::build(&framework, &extra_ignores, args.no_ignore, args.hidden);

    if args.show_ignore {
        println!("directree ignore rules ({} total):\n", ignorer.rules.len());
        for rule in &ignorer.rules { println!("  {}", rule); }
        return;
    }

    ignorer.load_local_ignore(&scan_root);

    let walker = Walker::new(&args, scan_root.clone(), framework.clone(), ignorer);
    let tree = walker.walk();

    if args.focus.is_some() {
        let rel = scan_root.strip_prefix(&root)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        println!("# focus: ./{}", rel);
        println!("# project root: {}/\n",
            root.file_name().and_then(|n| n.to_str()).unwrap_or("."));
    }

    let output = if args.ai {
        render_ai(&tree, &framework, &args)
    } else if args.flat || args.format == OutputFormat::Flat {
        render_flat(&tree, &scan_root, &args)
    } else if args.group || args.format == OutputFormat::Group {
        render_group(&tree, &scan_root, &args)
    } else if args.format == OutputFormat::Json {
        render_json(&tree)
    } else {
        render_tree(&tree, &args)
    };

    print!("{}", output);

    if args.stats && args.format != OutputFormat::Json {
        print!("{}", render_stats(&tree, &args));
    }
}
