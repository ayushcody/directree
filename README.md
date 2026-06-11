# directree

**AI-optimized project tree. Fast. Efficient. Context-ready.**

`directree` gives you a clean, signal-rich view of your project structure — purpose-built for pasting into AI context windows. Filters noise automatically, adds semantic annotations, and estimates token cost.

```
my-app/
├── src/  [source-root]
│   ├── app/  [app-router]
│   │   ├── layout.tsx  [layout]
│   │   ├── page.tsx  [page]
│   │   └── dashboard/
│   │       └── page.tsx  [page]
│   ├── components/  [ui-components]
│   │   └── Button.tsx  [ui-component]
│   └── lib/
│       └── supabase.ts  [db-client]
├── next.config.js  [config]
└── package.json  [manifest]
```

---

## Install

```bash
git clone https://github.com/ayush/directree
cd directree
cargo install --path .
```

Add aliases to your shell:

```bash
alias dt="directree"
alias dta="directree --ai"
alias dtf="directree --flat"
```

---

## Usage

```
directree [PATH] [OPTIONS]
```

| Command | What it does |
|---------|-------------|
| `directree` | Tree of current dir |
| `directree --ai` | AI context-ready XML output with annotations |
| `directree --ai --stats` | AI output + token cost estimate |
| `directree --flat` | Flat path list — cheapest token format |
| `directree --group` | Files grouped by type (source/config/test/asset) |
| `directree --format json` | Structured JSON tree |
| `directree src --depth 3` | Subdir, max 3 levels deep |
| `directree --only ts,tsx` | Only TypeScript files |
| `directree --focus src/components` | Zoom into subpath with breadcrumb |
| `directree --collapse` | Collapse repeated sibling patterns |
| `directree --show-ignore` | See all active ignore rules |
| `directree --stats` | File counts + estimated tokens |

---

## The `--ai` flag

```xml
<project_structure lang="typescript" framework="nextjs" files="8" est_tokens="420">
my-app/
├── src/  [source-root]
│   ├── app/  [app-router]
│   │   ├── layout.tsx  [layout]
│   │   └── page.tsx  [page]
│   └── lib/
│       └── supabase.ts  [db-client]
└── package.json  [manifest]
</project_structure>

tokens: ~420  ·  files: 8  ·  ignored: node_modules .git dist build
```

Paste this directly into Claude or GPT. The XML wrapper, lang/framework attributes, and role annotations give the model everything it needs to understand your structure immediately.

---

## Smart ignore

Built-in ignores that always apply:
`node_modules` `.git` `.DS_Store` `.idea` `.vscode` `.cache` `coverage` `.turbo` and more.

Framework-aware extras (auto-detected):
- **Next.js** → `.next/` `out/` `next-env.d.ts`
- **Rust** → `target/`
- **Python** → `__pycache__/` `.venv/` `dist/`
- **Go** → `vendor/`

Override with a `.directreeignore` file in your project root (same syntax as `.gitignore`).

Global config at `~/.directreerc`:

```toml
default_depth = 4
ignore = ["*.log", "scripts/migrations/", "**/*.generated.ts"]
```

---

## v0.2 Features

| Feature | Usage |
|---------|-------|
| **Clipboard** | `directree --ai --copy` → output straight to clipboard |
| **Git diff** | `directree --since HEAD~1` → only changed files |
| **Importance** | `directree --important` → ★ marks entry points, hot files, core infra |
| **Watch mode** | `directree --watch --ai` → live refresh on file save |
| **Init** | `directree init` → scaffold `.directreeignore` for your framework |
| **Completions** | `directree completions zsh` → tab completions for all flags |

### Shell setup (one-time)

```bash
# Zsh
source <(directree completions zsh)

# Bash
source <(directree completions bash)

# Fish — save to completions dir
directree completions fish > ~/.config/fish/completions/directree.fish
```

### Live AI context refresh workflow

```bash
# Terminal 1: keep AI context fresh as you code
directree --watch --ai --copy

# Paste once in Claude/GPT — re-paste after each file save (1 keystroke)
```

### Git diff workflow

```bash
directree --since HEAD~1 --ai     # what changed in last commit, AI-ready
directree --since main --ai       # your branch's changes vs main
directree --since HEAD --flat     # uncommitted changes as flat list
```



| Mode | 50-file project | 500-file project |
|------|----------------|-----------------|
| `--flat` | ~200 tokens | ~1,800 tokens |
| `--tree` | ~350 tokens | ~3,000 tokens |
| `--ai` | ~420 tokens | ~3,500 tokens |

**For huge monorepos:** `directree --flat --only ts,tsx` keeps context under 500 tokens.

---

## Project structure

```
src/
├── cli.rs        — Arg parser (zero deps, hand-rolled)
├── config.rs     — ~/.directreerc loader
├── detector.rs   — Framework detection + file role classifier
├── ignorer.rs    — Ignore engine (built-in + framework + .directreeignore + CLI)
├── walker.rs     — FS walker + sibling collapse engine
├── tree.rs       — TreeNode data structure + stats collector
└── render.rs     — All renderers: tree / ai / flat / group / json / stats
```

Zero external dependencies outside `serde`/`serde_json`. Single binary, no runtime.

---

*Built for the AI-native developer workflow.*
