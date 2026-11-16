//! Documentation checker: broken links, missing repository paths, personal data
//! and project memory stages. Runs with `cargo test`; see
//! docs/ai/DEVELOPMENT_GUIDE.md for what it checks.

use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Local notes (plans, specs): never checked, never committed.
const EXCLUDED: &str = "docs/superpowers/";
const MEMORY: &str = "docs/ai/PROJECT_MEMORY.md";
/// Documentation IP ranges (RFC 5737) and special addresses allowed in docs.
const DOC_IPS: [&str; 3] = ["192.0.2.", "198.51.100.", "203.0.113."];
const SPECIAL_IPS: [&str; 3] = ["127.0.0.1", "0.0.0.0", "255.255.255.255"];

/// What exists in the repository: the files git knows about (tracked or new,
/// not ignored) when `root` is a git top level, otherwise the disk.
struct Repo {
    root: PathBuf,
    files: Option<BTreeSet<String>>,
    /// Top-level names: a backticked path starting with one of them is checked.
    roots: BTreeSet<String>,
    /// Folders of the crate (`src`, `tests`...): `src/x.rs` means `rust/src/x.rs`.
    crate_dirs: BTreeSet<String>,
}

fn dir_names(dir: &Path, only_dirs: bool) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return BTreeSet::new();
    };
    entries
        .flatten()
        .filter(|e| !only_dirs || e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.') || n == ".github")
        .collect()
}

fn git_files(root: &Path) -> Option<BTreeSet<String>> {
    let top = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()
        .ok()?;
    let top = PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    if !top.is_dir() || top.canonicalize().ok()? != root.canonicalize().ok()? {
        return None;
    }
    let out = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .ok()?;
    out.status.success().then(|| {
        String::from_utf8_lossy(&out.stdout)
            .split('\0')
            .filter(|f| !f.is_empty())
            .map(String::from)
            .collect()
    })
}

impl Repo {
    fn new(root: &Path) -> Repo {
        let files = git_files(root);
        // From git when possible, so ignored folders (`rust/target`) do not count.
        let crate_dirs = match &files {
            Some(files) => files
                .iter()
                .filter_map(|f| f.strip_prefix("rust/")?.split_once('/'))
                .map(|(dir, _)| dir.to_string())
                .collect(),
            None => dir_names(&root.join("rust"), true),
        };
        Repo {
            root: root.to_path_buf(),
            files,
            roots: dir_names(root, false),
            crate_dirs,
        }
    }

    fn exists(&self, rel: &str) -> bool {
        if rel.contains('*') {
            let pattern = regex::escape(rel)
                .replace(r"\*\*/", "\0")
                .replace(r"\*", "[^/]*")
                .replace('\0', "(?:.*/)?");
            let pattern = format!("^{pattern}$");
            let glob = Regex::new(&pattern).unwrap();
            return match &self.files {
                Some(files) => files.iter().any(|f| glob.is_match(f)),
                None => walk(&self.root, &self.root)
                    .iter()
                    .any(|f| glob.is_match(f)),
            };
        }
        match &self.files {
            Some(files) => {
                let dir = format!("{rel}/");
                files.contains(rel)
                    || files
                        .range(dir.clone()..)
                        .next()
                        .is_some_and(|f| f.starts_with(&dir))
            }
            None => self.root.join(rel).exists(),
        }
    }
}

/// Every file under `dir`, as a path relative to `root` with `/` separators.
fn walk(root: &Path, dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(root, &path));
        } else {
            let rel = path.strip_prefix(root).unwrap();
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out
}

/// The checked documents: the root guides, `packaging/README.md` and every
/// Markdown file under `docs/` except the local notes.
fn documents(root: &Path) -> Vec<String> {
    let mut docs: Vec<String> = [
        "README.md",
        "README.fr.md",
        "README.es.md",
        "README.pt.md",
        "README.it.md",
        "README.de.md",
        "README.ru.md",
        "CONTRIBUTING.md",
        "AGENTS.md",
        "packaging/README.md",
    ]
    .iter()
    .map(|s| s.to_string())
    .filter(|f| root.join(f).is_file())
    .collect();
    let mut under_docs: Vec<String> = walk(root, &root.join("docs"))
        .into_iter()
        .filter(|f| f.ends_with(".md") && !f.starts_with(EXCLUDED))
        .collect();
    under_docs.sort();
    docs.extend(under_docs);
    docs
}

/// GitHub heading anchors of a Markdown file (code blocks skipped; repeated
/// headings get `-1`, `-2`...).
fn anchors(path: &Path) -> BTreeSet<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut out = BTreeSet::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        let Some(title) = line.strip_prefix('#').map(|t| t.trim_start_matches('#')) else {
            continue;
        };
        if in_code || !title.starts_with(' ') {
            continue;
        }
        let slug: String = title
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        let mut unique = slug.clone();
        let mut n = 0;
        while !out.insert(unique.clone()) {
            n += 1;
            unique = format!("{slug}-{n}");
        }
    }
    out
}

struct Rules {
    link: Regex,
    ref_link: Regex,
    ticks: Regex,
    scheme: Regex,
    line_suffix: Regex,
    email: Regex,
    personal: Vec<(Regex, &'static str)>,
    ipv4: Regex,
    stage: Regex,
}

impl Rules {
    fn new() -> Rules {
        let re = |s: &str| Regex::new(s).unwrap();
        Rules {
            link: re(r#"\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)"#),
            ref_link: re(r"^\s*\[[^\]]+\]:\s*(\S+)"),
            ticks: re(r"`([^`\n]+)`"),
            scheme: re(r"^([a-z][a-z0-9+.-]*:|#)"),
            line_suffix: re(r"(:\d+)+(-\d+)?$"),
            email: re(r"[A-Za-z0-9._%+-]+@([A-Za-z0-9-]+\.)+[A-Za-z]{2,}"),
            personal: vec![
                // A drive path, unless it is a placeholder like `C:/<game>`.
                (re(r"(?:^|[^%\w>])[A-Za-z]:[\\/][^<]"), "drive path"),
                (re(r"/home/"), "/home path"),
                (re(r"/mnt/"), "/mnt path"),
                (re(r"/Users/"), "/Users path"),
                (
                    re(r"(?:^|[^0-9A-Za-z])[0-9a-fA-F]{32,}(?:$|[^0-9A-Za-z])"),
                    "possible key",
                ),
            ],
            ipv4: re(r"(?:^|[^\d.])((\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3}))(\.\d|\d)?"),
            stage: re(r"^## \d{4}-\d{2}-\d{2} — \S"),
        }
    }

    fn links(&self, root: &Path, doc: &str, line: &str) -> Vec<String> {
        let dir = root.join(doc);
        let dir = dir.parent().unwrap();
        let targets = self
            .link
            .captures_iter(line)
            .chain(self.ref_link.captures_iter(line));
        let mut out = Vec::new();
        for target in targets.map(|c| c[1].to_string()) {
            if !target.starts_with('#') && self.scheme.is_match(&target) {
                continue;
            }
            let (file, anchor) = target.split_once('#').unwrap_or((&target, ""));
            let path = if file.is_empty() {
                root.join(doc)
            } else {
                dir.join(file)
            };
            if !path.exists() {
                out.push(format!("broken link: {target}"));
            } else if !anchor.is_empty()
                && path.extension().is_some_and(|e| e == "md")
                && !anchors(&path).contains(anchor)
            {
                out.push(format!("broken anchor: {target}"));
            }
        }
        out
    }

    fn paths(&self, repo: &Repo, line: &str) -> Vec<String> {
        let mut out = Vec::new();
        for c in self.ticks.captures_iter(line) {
            let token = self.line_suffix.replace(c[1].trim(), "");
            let token = token.trim_end_matches('/');
            if token.contains(' ')
                || token.contains('<')
                || !token.contains('/')
                || format!("{token}/").starts_with(EXCLUDED)
            {
                continue;
            }
            let first = token.split('/').next().unwrap();
            let ok = if repo.roots.contains(first) {
                repo.exists(token)
            } else if repo.crate_dirs.contains(first) {
                repo.exists(&format!("rust/{token}"))
            } else {
                continue;
            };
            if !ok {
                out.push(format!("missing path: {token}"));
            }
        }
        out
    }

    fn personal(&self, line: &str) -> Vec<String> {
        let emails = self.email.find_iter(line).map(|m| m.as_str());
        let mut out: Vec<String> = emails
            .filter(|e| !e.starts_with("noreply@") && !e.contains("@example."))
            .map(|e| format!("personal data (email {e})"))
            .collect();
        out.extend(
            self.personal
                .iter()
                .filter(|(rx, _)| rx.is_match(line))
                .map(|(_, what)| format!("personal data ({what})")),
        );
        for c in self.ipv4.captures_iter(line) {
            let ip = &c[1];
            let valid = (2..=5).all(|i| c[i].parse::<u16>().unwrap() <= 255);
            if valid
                && c.get(6).is_none()
                && !SPECIAL_IPS.contains(&ip)
                && !DOC_IPS.iter().any(|r| ip.starts_with(r))
            {
                out.push(format!(
                    "personal data (IPv4 {ip}; use 192.0.2.x, 198.51.100.x or 203.0.113.x)"
                ));
            }
        }
        out
    }

    fn memory(&self, text: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        let mut stage: Option<(usize, String, usize)> = None;
        let close = |stage: &Option<(usize, String, usize)>, out: &mut Vec<(usize, String)>| {
            if let Some((start, name, bullets)) = stage
                && *bullets > 3
            {
                out.push((
                    *start,
                    format!("stage with {bullets} bullets (3 at most): {name}"),
                ));
            }
        };
        for (n, line) in text.lines().enumerate() {
            if let Some(name) = line.strip_prefix("## ") {
                close(&stage, &mut out);
                stage = Some((n + 1, name.trim().to_string(), 0));
                if !self.stage.is_match(line) {
                    out.push((n + 1, "stage heading is not `## YYYY-MM-DD — topic`".into()));
                }
            } else if let Some((_, _, bullets)) = &mut stage
                && (line.starts_with("- ") || line.starts_with("* "))
            {
                *bullets += 1;
            }
        }
        close(&stage, &mut out);
        out
    }
}

/// Problems found, as `path:line: message`; empty when all is well.
fn check(root: &Path) -> Vec<String> {
    let repo = Repo::new(root);
    let rules = Rules::new();
    let mut problems = Vec::new();
    for doc in documents(root) {
        let text = std::fs::read_to_string(root.join(&doc)).unwrap();
        let personal = doc == "AGENTS.md" || doc.starts_with("docs/");
        for (n, line) in text.lines().enumerate() {
            let mut found = rules.links(root, &doc, line);
            found.extend(rules.paths(&repo, line));
            if personal {
                found.extend(rules.personal(line));
            }
            problems.extend(found.into_iter().map(|m| format!("{doc}:{}: {m}", n + 1)));
        }
        if doc == MEMORY {
            problems.extend(
                rules
                    .memory(&text)
                    .into_iter()
                    .map(|(n, m)| format!("{doc}:{n}: {m}")),
            );
        }
    }
    problems
}

// ---------------------------------------------------------------- tests

fn repo(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }
    dir
}

fn problems(files: &[(&str, &str)]) -> Vec<String> {
    check(repo(files).path())
}

#[test]
fn the_repository_documentation_is_consistent() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let found = check(&root);
    assert!(
        found.is_empty(),
        "documentation problems:\n{}",
        found.join("\n")
    );
}

#[test]
fn clean_docs_have_no_problems() {
    let found = problems(&[
        ("rust/src/lib.rs", ""),
        (
            "docs/a.md",
            "# Top\n\nSee [b](b.md#code-and-paths), [web](https://example.org) and [top](#top).\n",
        ),
        (
            "docs/b.md",
            "## Code and paths\n\nCode in `rust/src/lib.rs:3:5`, `src/lib.rs`, `rust/src/*.rs` and `docs/**/*.md`.\n",
        ),
        (
            "AGENTS.md",
            "Server 192.0.2.10:27960, local 127.0.0.1, `C:/<game>`, %APPDATA%\\x, noreply@anthropic.com, a@example.org.\n",
        ),
    ]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn broken_relative_links_are_reported() {
    let found = problems(&[(
        "docs/a.md",
        "[x](missing.md) [y](../nope.md#part \"title\")\n[ref]: gone.md\n",
    )]);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].starts_with("docs/a.md:1: broken link: missing.md"));
}

#[test]
fn broken_anchors_are_reported() {
    let found = problems(&[
        (
            "docs/a.md",
            "# Title\n\n```sh\n# not a heading\n```\n[x](b.md#gone) [y](#nope) [z](#not-a-heading)\n",
        ),
        ("docs/b.md", "# Here\n"),
    ]);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].contains("broken anchor: b.md#gone"));
}

#[test]
fn missing_repository_paths_are_reported() {
    let found = problems(&[
        ("rust/src/lib.rs", ""),
        ("docs/x.md", ""),
        (
            "README.md",
            "`rust/src/gone.rs`, `src/gone.rs`, `docs/nothing/` and `rust/src/*.py`\n",
        ),
    ]);
    assert_eq!(found.len(), 4, "{found:?}");
}

#[test]
fn paths_outside_the_repository_are_not_checked() {
    let found = problems(&[(
        "README.md",
        "`System/UT.exe`, `a b/c`, `docs/<id>.md`, `https://x.org/y` and `host:port/x`\n",
    )]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn local_notes_are_excluded() {
    let found = problems(&[(
        "docs/superpowers/plan.md",
        "[x](missing.md) `docs/gone.md`\n",
    )]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn mentions_of_local_notes_are_not_missing_paths() {
    let found = problems(&[
        ("docs/x.md", ""),
        (
            "AGENTS.md",
            "Never commit `docs/superpowers/` or `docs/superpowers`.\n",
        ),
    ]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn personal_data_is_reported_in_agents_and_docs() {
    let line = "D:\\Games, /home/someone, /mnt/c, /Users/someone, someone@mail.test, 10.1.2.3 and 0123456789abcdef0123456789abcdef\n";
    let found = problems(&[("docs/a.md", line), ("AGENTS.md", line)]);
    assert_eq!(found.len(), 14, "{found:?}");
}

#[test]
fn personal_data_is_not_checked_in_readme() {
    let found = problems(&[("README.md", "master at 10.1.2.3\n")]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn memory_stages_are_checked() {
    let memory = "# Memory\n\n## 2026-01-02 — good\n- a\n- b\n- c\n\n\
                  ## 2026-01-03 — too long\n- a\n- b\n- c\n- d\n\n## no date\n- a\n";
    let found = problems(&[("docs/ai/PROJECT_MEMORY.md", memory)]);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].contains("4 bullets"));
    assert!(found[1].contains("YYYY-MM-DD"));
}

/// The catalog figures quoted in the docs match `games.json`.
#[test]
fn catalog_counts_in_the_docs_match_games_json() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let text = std::fs::read_to_string(root.join("rust/src/games.json")).unwrap();
    let games: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    let masters: Vec<&serde_json::Value> = games
        .iter()
        .flat_map(|g| g["masters"].as_array().unwrap())
        .collect();
    let protocols: BTreeSet<&str> = games
        .iter()
        .map(|g| g["protocol"].as_str().unwrap())
        .collect();
    let kinds: BTreeSet<&str> = masters
        .iter()
        .map(|m| m["kind"].as_str().unwrap())
        .collect();
    let (g, p, k, e) = (games.len(), protocols.len(), kinds.len(), masters.len());
    let expected = [
        (
            "docs/ai/SYSTEM_SUMMARY.md",
            format!("{g} games, {p} query protocols, {k} master kinds, {e} master entries"),
        ),
        (
            "docs/architecture/overview.md",
            format!("{g} games, {p} protocols, {k} master kinds"),
        ),
    ];
    for (doc, figures) in expected {
        let text = std::fs::read_to_string(root.join(doc)).unwrap();
        assert!(text.contains(&figures), "{doc} should say \"{figures}\"");
    }
}
