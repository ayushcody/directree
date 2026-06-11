# directree — Product Requirements Document

**Version:** 0.1.0  
**Author:** Ayush  
**Status:** Implemented

---

## 1. Problem Statement

When giving an AI assistant context about a codebase, developers face three friction points:

1. **Noise** — `tree` outputs `node_modules/`, `.git/`, `dist/`, and hundreds of irrelevant files. Pasting this raw costs tokens and confuses the model.
2. **No semantics** — `tree` shows file names, not what they *do*. An AI reading `layout.tsx` gets no hint it's the root layout. Reading `supabase.ts` gets no hint it's the DB client.
3. **Wrong format** — `tree`'s box-drawing output is fine for humans, but AI context windows benefit from structured, annotated, or compact formats.

**directree** solves all three. It is a CLI tool purpose-built for producing AI-ready project structure output — fast, filtered, and semantically annotated.

---

## 2. Goals

| Goal | Description |
|------|-------------|
| **Signal-to-noise** | Zero-config filtering of build artifacts, dependencies, hidden files, and framework-specific noise |
| **Semantic awareness** | Annotate files with roles (`[layout]`, `[api-route]`, `[db-client]`) so AI knows what it's looking at |
| **AI-first output** | `--ai` flag emits a structured XML block optimised for pasting into AI prompts |
| **Token efficiency** | Multiple output modes from verbose tree to ultra-compact flat list; always shows estimated token cost |
| **Speed** | Sub-10ms for typical projects; streaming output starts before full scan completes |
| **Zero dependencies** | Single statically-linked binary, `cargo install`-able, no runtime requirements |

---

## 3. Non-Goals

- Not a replacement for `find`, `ls`, or `fd` for day-to-day shell use
- Not a file content reader (shows structure only)
- Not a GUI or TUI — pure terminal output
- Not a code analysis tool (no AST parsing, no LOC counting)

---

## 4. User Personas

**Primary: Ayush / AI-heavy developer**
- Workflow: open project → run `directree --ai` → paste into Claude/GPT → ask questions
- Pain: wastes 10-30 seconds manually cleaning tree output before pasting
- Need: one command, clean output, ready to paste

**Secondary: Team lead onboarding new dev**
- Workflow: `directree --group --stats` → share in Slack as project overview
- Need: grouped, readable, shows what kind of project this is at a glance

**Tertiary: CI/automation**
- Workflow: `directree --format json | jq ...` in scripts
- Need: stable, machine-readable output

---

## 5. Features

### 5.1 Core — Output Modes

| Mode | Flag | Description | Token cost |
|------|------|-------------|------------|
| **Tree** | (default) | Classic box-drawing tree, colored by file type | Medium |
| **AI** | `--ai` | `<project_structure>` XML with semantic annotations | Medium+ |
| **Flat** | `--flat` | One relative path per line | Cheapest |
| **Group** | `--group` | Files grouped: source / config / test / asset / docs | Medium |
| **JSON** | `--format json` | Full structured JSON tree | High (but machine-readable) |

### 5.2 Filtering

| Feature | Flag | Notes |
|---------|------|-------|
| Built-in ignore | (always) | `node_modules`, `.git`, `.DS_Store`, build dirs, etc. |
| Framework-aware ignore | (auto) | Detects Next.js/Rust/Python/Go and adds relevant patterns |
| Extension filter | `--only ts,tsx` | Show only matching file types; empty dirs auto-removed |
| Depth limit | `--depth N` | Truncate with `… N more` hint so nothing is silently dropped |
| Hidden files | `--hidden` | Off by default; opt in to show dotfiles |
| Local override | `.directreeignore` | Works like `.gitignore`; project-level custom patterns |
| Global config | `~/.directreerc` | TOML-like: `ignore`, `default_depth` |
| CLI override | `--ignore pats` | Comma-separated extra patterns per invocation |
| Disable built-ins | `--no-ignore` | Show everything (useful for debugging) |
| Show rules | `--show-ignore` | Print all active ignore rules and exit |

### 5.3 Focus Mode

```
directree --focus src/components
```

- Trees only the specified subpath
- Prints a breadcrumb header: `# focus: ./src/components` + `# project root: my-app/`
- Lets you provide AI with a zoomed view without re-scanning the whole project

### 5.4 Collapse Mode

```
directree --collapse
```

Detects sibling directories with identical internal shape (same extension mix). If 4+ match, collapses to `Button ×12 (similar structure)`. Prevents AI context bloat in component-heavy projects.

### 5.5 Stats

```
directree --stats
```

Appended footer showing:
- File count, dir count, source files, test files
- Top 8 extensions by count
- **Estimated token cost** of the tree output (~chars × 1.3 / 4)

### 5.6 Framework Detection

Reads `package.json`, `Cargo.toml`, `go.mod`, `requirements.txt`, `pyproject.toml`, `pom.xml` to detect:

| Framework | Detected by | Extra ignores |
|-----------|-------------|---------------|
| Next.js   | `package.json` → `"next"` | `.next/`, `out/`, `next-env.d.ts` |
| React     | `package.json` → `"react"` | `dist/`, `build/` |
| Vue       | `package.json` → `"vue"` | `dist/`, `.nuxt/` |
| Svelte    | `package.json` → `"svelte"` | `.svelte-kit/` |
| Astro     | `package.json` → `"astro"` | `dist/`, `.astro/` |
| Vite      | `package.json` → `"vite"` | `dist/` |
| Rust      | `Cargo.toml` | `target/` |
| Python    | `requirements.txt` / `pyproject.toml` | `__pycache__/`, `.venv/`, `dist/` |
| Go        | `go.mod` | `vendor/` |
| Java      | `pom.xml` / `build.gradle` | `target/`, `build/`, `.gradle/` |
| .NET      | `*.csproj` / `*.sln` | `bin/`, `obj/` |

### 5.7 File Role Annotations (AI mode)

Files are classified into roles and annotated in `--ai` and `--group` modes:

| Role | Example files | Annotation |
|------|--------------|------------|
| Layout | `layout.tsx` | `[layout]` |
| Page | `page.tsx`, `index.tsx` | `[page]` |
| Component | `src/components/*.tsx` | `[ui-component]` |
| Hook | `src/hooks/use*.ts` | `[hook]` |
| Lib/Util | `src/lib/`, `src/utils/` | `[lib]`, `[util]` |
| DB client | `supabase.ts`, `prisma.ts` | `[db-client]` |
| API route | `src/app/api/**` | `[api-route]` |
| Middleware | `middleware.ts` | `[middleware]` |
| Store | `src/store/`, `context/` | `[store]` |
| Types | `*.d.ts`, `src/types/` | `[types]` |
| Entry | `main.rs`, `main.py`, `__main__.py` | `[entry]` |
| Config | `next.config.js`, `tailwind.config.ts` | `[config]` |
| Env | `.env*` | `[env]` |
| Lockfile | `Cargo.lock`, `yarn.lock` | `[lockfile]` |
| Test | `*.test.ts`, `*.spec.ts` | `[test]` |

Directory annotations:

| Dir name | Annotation |
|----------|------------|
| `components/` | `[ui-components]` |
| `app/` | `[app-router]` |
| `pages/` | `[pages-router]` |
| `api/` | `[api-routes]` |
| `hooks/` | `[react-hooks]` |
| `store/` | `[state]` |
| `migrations/` | `[db-migrations]` |
| `src/` | `[source-root]` |

---

## 6. Output Examples

### Default tree
```
my-app/
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx
│   │   └── dashboard/
│   │       └── page.tsx
│   ├── components/
│   │   ├── Button.tsx
│   │   └── Card.tsx
│   └── lib/
│       └── supabase.ts
├── next.config.js
├── tailwind.config.ts
└── package.json
```

### `--ai` mode
```xml
<project_structure lang="typescript" framework="nextjs" files="8" est_tokens="420">
my-app/
├── src/  [source-root]
│   ├── app/  [app-router]
│   │   ├── layout.tsx  [layout]
│   │   ├── page.tsx  [page]
│   │   └── dashboard/
│   │       └── page.tsx  [page]
│   ├── components/  [ui-components]
│   │   ├── Button.tsx  [ui-component]
│   │   └── Card.tsx  [ui-component]
│   └── lib/  [lib]
│       └── supabase.ts  [db-client]
├── next.config.js  [config]
├── tailwind.config.ts  [config]
└── package.json  [manifest]
</project_structure>

tokens: ~420  ·  files: 8  ·  ignored: node_modules .git dist build
```

### `--flat` mode
```
# directree --flat  (8 paths)
src/app/layout.tsx
src/app/page.tsx
src/app/dashboard/page.tsx
src/components/Button.tsx
src/components/Card.tsx
src/lib/supabase.ts
next.config.js
package.json

8 paths  ·  ~110 tokens
```

### `--group` mode
```
── source ───────────────────────────────
src/app/layout.tsx  [layout]
src/app/page.tsx  [page]
src/components/Button.tsx  [ui-component]
src/lib/supabase.ts  [db-client]

── config ───────────────────────────────
next.config.js  [config]
tailwind.config.ts  [config]
package.json  [manifest]
```

---

## 7. Architecture

```
main.rs
  ├── cli.rs          — Arg parsing (hand-rolled, zero deps)
  ├── config.rs       — ~/.directreerc loader
  ├── detector.rs     — Framework detection + file role classification
  ├── ignorer.rs      — Ignore rules engine (built-in + framework + local + CLI)
  ├── walker.rs       — FS walker + collapse engine
  ├── tree.rs         — TreeNode data structure + stats
  └── render.rs       — All output renderers (tree/ai/flat/group/json/stats)
```

**Key design decisions:**
- **Zero external deps** (except `serde`/`serde_json` for JSON output) — avoids version hell, stays fast to compile
- **Hand-rolled glob matching** — trie-style `*`/`?` wildcard matcher, O(pattern × path_length)
- **ANSI colors via escape codes** — no `colored` crate, auto-disabled when stdout is not a TTY
- **Single pass walk** — dirs and files collected in one `read_dir` call, sorted, then rendered
- **Streaming-friendly** — output is printed as each node is processed; no full-tree buffering required for tree/flat modes

---

## 8. Installation

### From source (Rust required)
```bash
git clone https://github.com/ayush/directree
cd directree
cargo install --path .
```

### Binary in PATH
```bash
# After cargo install:
directree --version
```

### Alias suggestions
```bash
# ~/.zshrc or ~/.bashrc
alias dt="directree"
alias dta="directree --ai"
alias dtf="directree --flat"
alias dts="directree --stats"
```

---

## 9. Token Cost Reference

| Mode | Typical small project (50 files) | Large project (500 files) |
|------|----------------------------------|--------------------------|
| `--flat` | ~200 tokens | ~1,800 tokens |
| `--tree` | ~350 tokens | ~3,000 tokens |
| `--ai` | ~420 tokens | ~3,500 tokens |
| `--group` | ~380 tokens | ~3,200 tokens |
| `--format json` | ~900 tokens | ~8,000 tokens |

**Tip:** For large projects, combine `--flat --only ts,tsx,rs` to get a sub-300 token context even on monorepos.

---

## 10. Roadmap (v0.2+)

- [ ] `--copy` flag — pipe directly to clipboard (pbcopy / xclip / wl-copy)
- [ ] `--since <git-ref>` — show only files changed since a commit
- [ ] `--important` — heuristic scoring to surface "important" files (high import count, recently modified)
- [ ] `directree init` — scaffold `.directreeignore` with smart defaults for detected framework
- [ ] Shell completions (zsh, bash, fish)
- [ ] `--watch` — re-output on filesystem change (for live context refresh)
- [ ] WASM build for browser/web playground

---

*directree — built for the AI-native developer workflow.*
