use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Framework {
    NextJs,
    React,
    Vue,
    Svelte,
    Astro,
    Vite,
    Rust,
    Python,
    Go,
    Java,
    DotNet,
    Node,
    Unknown,
}

impl Framework {
    pub fn name(&self) -> &'static str {
        match self {
            Framework::NextJs => "nextjs",
            Framework::React => "react",
            Framework::Vue => "vue",
            Framework::Svelte => "svelte",
            Framework::Astro => "astro",
            Framework::Vite => "vite",
            Framework::Rust => "rust",
            Framework::Python => "python",
            Framework::Go => "go",
            Framework::Java => "java",
            Framework::DotNet => "dotnet",
            Framework::Node => "node",
            Framework::Unknown => "unknown",
        }
    }

    pub fn lang(&self) -> &'static str {
        match self {
            Framework::NextJs | Framework::React | Framework::Vue
            | Framework::Svelte | Framework::Astro | Framework::Vite
            | Framework::Node => "typescript",
            Framework::Rust => "rust",
            Framework::Python => "python",
            Framework::Go => "go",
            Framework::Java => "java",
            Framework::DotNet => "csharp",
            Framework::Unknown => "unknown",
        }
    }

    /// Extra directories/files to ignore for this framework
    pub fn extra_ignores(&self) -> Vec<&'static str> {
        match self {
            Framework::NextJs => vec![".next", "out", "next-env.d.ts", ".vercel"],
            Framework::React | Framework::Vite => vec!["dist", "build"],
            Framework::Vue => vec!["dist", ".nuxt", ".output"],
            Framework::Svelte => vec![".svelte-kit", "build"],
            Framework::Astro => vec!["dist", ".astro"],
            Framework::Rust => vec!["target"],
            Framework::Python => vec!["__pycache__", ".venv", "venv", "env", ".eggs",
                                      "*.egg-info", "dist", "build", ".pytest_cache",
                                      ".mypy_cache", ".ruff_cache"],
            Framework::Go => vec!["vendor"],
            Framework::Java => vec!["target", "build", ".gradle", "out", "*.class"],
            Framework::DotNet => vec!["bin", "obj", ".vs"],
            _ => vec![],
        }
    }
}

pub fn detect_framework(root: &Path) -> Framework {
    // Check package.json for JS/TS frameworks
    let pkg = root.join("package.json");
    if pkg.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg) {
            if content.contains("\"next\"") {
                return Framework::NextJs;
            }
            if content.contains("\"astro\"") {
                return Framework::Astro;
            }
            if content.contains("\"@sveltejs/kit\"") || content.contains("\"svelte\"") {
                return Framework::Svelte;
            }
            if content.contains("\"vue\"") {
                return Framework::Vue;
            }
            if content.contains("\"vite\"") {
                return Framework::Vite;
            }
            if content.contains("\"react\"") {
                return Framework::React;
            }
            return Framework::Node;
        }
    }
    if root.join("Cargo.toml").exists() {
        return Framework::Rust;
    }
    if root.join("go.mod").exists() {
        return Framework::Go;
    }
    if root.join("requirements.txt").exists()
        || root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
    {
        return Framework::Python;
    }
    if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
        return Framework::Java;
    }
    if root.join("*.csproj").exists() || root.join("*.sln").exists() {
        return Framework::DotNet;
    }
    Framework::Unknown
}

// ── File type / role annotation ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileRole {
    // Source
    SourceMain,
    Layout,
    Page,
    Component,
    Hook,
    Util,
    Lib,
    DbClient,
    ApiRoute,
    Middleware,
    Store,
    Types,
    // Config
    Config,
    Manifest,
    EnvFile,
    Lockfile,
    Dockerfile,
    CiConfig,
    // Test
    Test,
    TestUtil,
    // Asset
    Asset,
    Style,
    // Docs
    Readme,
    Docs,
    // Generic
    Source,
    #[default]
    Unknown,
}

impl FileRole {
    pub fn annotation(&self) -> Option<&'static str> {
        match self {
            FileRole::Layout => Some("layout"),
            FileRole::Page => Some("page"),
            FileRole::Component => Some("ui-component"),
            FileRole::Hook => Some("hook"),
            FileRole::Util => Some("util"),
            FileRole::Lib => Some("lib"),
            FileRole::DbClient => Some("db-client"),
            FileRole::ApiRoute => Some("api-route"),
            FileRole::Middleware => Some("middleware"),
            FileRole::Store => Some("store"),
            FileRole::Types => Some("types"),
            FileRole::Config => Some("config"),
            FileRole::Manifest => Some("manifest"),
            FileRole::EnvFile => Some("env"),
            FileRole::Lockfile => Some("lockfile"),
            FileRole::Dockerfile => Some("docker"),
            FileRole::CiConfig => Some("ci"),
            FileRole::Test => Some("test"),
            FileRole::TestUtil => Some("test-util"),
            FileRole::Style => Some("styles"),
            FileRole::Readme => Some("readme"),
            FileRole::Docs => Some("docs"),
            FileRole::SourceMain => Some("entry"),
            _ => None,
        }
    }

    pub fn category(&self) -> FileCategory {
        match self {
            FileRole::Config | FileRole::Manifest | FileRole::EnvFile
            | FileRole::Lockfile | FileRole::Dockerfile | FileRole::CiConfig => FileCategory::Config,
            FileRole::Test | FileRole::TestUtil => FileCategory::Test,
            FileRole::Asset | FileRole::Style => FileCategory::Asset,
            FileRole::Readme | FileRole::Docs => FileCategory::Docs,
            _ => FileCategory::Source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileCategory {
    Source,
    Config,
    Test,
    Asset,
    Docs,
}

impl FileCategory {
    pub fn label(&self) -> &'static str {
        match self {
            FileCategory::Source => "source",
            FileCategory::Config => "config",
            FileCategory::Test => "tests",
            FileCategory::Asset => "assets",
            FileCategory::Docs => "docs",
        }
    }
}

pub fn classify_file(path: &Path, framework: &Framework) -> FileRole {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let stem = path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Path segments for context
    let path_str = path.to_string_lossy().to_lowercase();
    let path_str = path_str.replace('\\', "/");

    // Lockfiles
    if matches!(name.as_str(), "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml"
                | "cargo.lock" | "poetry.lock" | "pipfile.lock" | "go.sum") {
        return FileRole::Lockfile;
    }

    // Env
    if name.starts_with(".env") {
        return FileRole::EnvFile;
    }

    // Dockerfile
    if name == "dockerfile" || name.starts_with("dockerfile.") {
        return FileRole::Dockerfile;
    }

    // CI configs
    if path_str.contains("/.github/workflows/")
        || path_str.contains("/.circleci/")
        || name == ".gitlab-ci.yml"
        || name == "jenkinsfile"
    {
        return FileRole::CiConfig;
    }

    // Readme / docs
    if stem == "readme" {
        return FileRole::Readme;
    }
    if matches!(ext.as_str(), "md" | "mdx" | "rst" | "txt")
        && (path_str.contains("/docs/") || path_str.contains("/doc/"))
    {
        return FileRole::Docs;
    }

    // Tests
    if name.contains(".test.") || name.contains(".spec.") || name.ends_with("_test.rs")
        || name.ends_with("_test.go") || name.ends_with("_test.py")
        || path_str.contains("/__tests__/") || path_str.contains("/tests/")
        || path_str.contains("/test/") || path_str.contains("/spec/")
    {
        return FileRole::Test;
    }
    if matches!(name.as_str(), "jest.config.js" | "jest.config.ts" | "vitest.config.ts"
                | "vitest.config.js" | "playwright.config.ts" | "cypress.config.ts") {
        return FileRole::TestUtil;
    }

    // Styles
    if matches!(ext.as_str(), "css" | "scss" | "sass" | "less" | "styl") {
        return FileRole::Style;
    }

    // Assets
    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp"
                | "ico" | "woff" | "woff2" | "ttf" | "eot" | "mp4" | "mp3"
                | "pdf" | "zip" | "tar" | "gz") {
        return FileRole::Asset;
    }

    // Config files
    if matches!(ext.as_str(), "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf")
        && !path_str.contains("/src/")
    {
        if name == "package.json" {
            return FileRole::Manifest;
        }
        return FileRole::Config;
    }
    if matches!(name.as_str(), "tailwind.config.ts" | "tailwind.config.js"
                | "next.config.js" | "next.config.ts" | "next.config.mjs"
                | "vite.config.ts" | "vite.config.js" | "tsconfig.json"
                | "eslint.config.js" | ".eslintrc.js" | ".eslintrc.json"
                | ".prettierrc" | ".babelrc" | "babel.config.js"
                | "webpack.config.js" | "rollup.config.js" | "astro.config.mjs"
                | "svelte.config.js" | "nuxt.config.ts")
    {
        return FileRole::Config;
    }

    // Source role detection — framework-aware
    match framework {
        Framework::NextJs | Framework::Astro => {
            if stem == "layout" { return FileRole::Layout; }
            if stem == "page" || stem == "index" { return FileRole::Page; }
            if path_str.contains("/api/") || path_str.contains("/route") {
                return FileRole::ApiRoute;
            }
            if path_str.contains("/middleware") { return FileRole::Middleware; }
            if path_str.contains("/components/") { return FileRole::Component; }
            if path_str.contains("/hooks/") || stem.starts_with("use") {
                return FileRole::Hook;
            }
            if path_str.contains("/store/") || path_str.contains("/stores/")
                || path_str.contains("/context/")
            {
                return FileRole::Store;
            }
            if path_str.contains("/lib/") || path_str.contains("/utils/") {
                if name.contains("supabase") || name.contains("prisma") || name.contains("db") {
                    return FileRole::DbClient;
                }
                return FileRole::Lib;
            }
            if path_str.contains("/types/") || stem == "types" || stem == "globals"
                || ext == "d.ts"
            {
                return FileRole::Types;
            }
        }
        Framework::Rust => {
            if stem == "main" { return FileRole::SourceMain; }
            if stem == "lib" { return FileRole::Lib; }
            if path_str.contains("/bin/") { return FileRole::SourceMain; }
        }
        Framework::Python => {
            if stem == "__main__" || stem == "main" || stem == "app" || stem == "wsgi" {
                return FileRole::SourceMain;
            }
            if path_str.contains("/models/") || path_str.contains("/schemas/") {
                return FileRole::Types;
            }
            if path_str.contains("/utils/") || path_str.contains("/helpers/") {
                return FileRole::Util;
            }
            if path_str.contains("/db/") || path_str.contains("/database/")
                || name.contains("database") || name.contains("models")
            {
                return FileRole::DbClient;
            }
        }
        _ => {}
    }

    // Generic source
    if matches!(ext.as_str(), "rs" | "go" | "py" | "js" | "ts" | "jsx" | "tsx"
                | "vue" | "svelte" | "java" | "kt" | "cs" | "cpp" | "c" | "h"
                | "rb" | "php" | "swift") {
        return FileRole::Source;
    }

    FileRole::Unknown
}
