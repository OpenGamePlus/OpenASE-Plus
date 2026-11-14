//! Interface texts: English in the source, translated through tables.
//!
//! Source code keeps English literals (`tr("Refresh")`); `tr` returns the
//! text of the active language from `src/locales/<code>.tsv` (one line per
//! text, `English<TAB>translation`, with `\t`, `\n` and `\\` escapes) and
//! the English input when a line is missing. Strings with values use `{}`
//! placeholders filled by [`fill`], one entry per whole sentence.
//! `tests/i18n.rs` fails when a `tr("...")` literal is missing from a table.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

/// Interface language chosen in Options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Language {
    /// The operating system's language when it is one of ours, else English.
    #[default]
    System,
    English,
    French,
    Spanish,
    Portuguese,
    Italian,
    German,
    Russian,
}

/// Translated languages: (language, ISO 639-1 code, table).
const TABLES: [(Language, &str, &str); 6] = [
    (Language::French, "fr", include_str!("locales/fr.tsv")),
    (Language::Spanish, "es", include_str!("locales/es.tsv")),
    (Language::Portuguese, "pt", include_str!("locales/pt.tsv")),
    (Language::Italian, "it", include_str!("locales/it.tsv")),
    (Language::German, "de", include_str!("locales/de.tsv")),
    (Language::Russian, "ru", include_str!("locales/ru.tsv")),
];

impl Language {
    pub const ALL: [Language; 8] = [
        Language::System,
        Language::English,
        Language::French,
        Language::Spanish,
        Language::Portuguese,
        Language::Italian,
        Language::German,
        Language::Russian,
    ];

    /// Name shown in the language list (each in its own language).
    pub fn label(self) -> &'static str {
        match self {
            Language::System => noop("System"),
            Language::English => "English",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Portuguese => "Português",
            Language::Italian => "Italiano",
            Language::German => "Deutsch",
            Language::Russian => "Русский",
        }
    }

    /// The language `tr` uses: `System` resolved, English for untranslated.
    pub fn resolved(self) -> Language {
        match self {
            Language::System => system_language()
                .and_then(|code| from_code(&code))
                .unwrap_or(Language::English),
            other => other,
        }
    }

    pub fn is_french(self) -> bool {
        self.resolved() == Language::French
    }
}

/// The language of an ISO code or locale name (`es`, `pt_BR.UTF-8`, `de-AT`).
pub fn from_code(code: &str) -> Option<Language> {
    let code = code.get(..2)?.to_ascii_lowercase();
    if code == "en" {
        return Some(Language::English);
    }
    TABLES
        .iter()
        .find(|(_, c, _)| *c == code)
        .map(|(l, _, _)| *l)
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// One parsed table: English text -> translation (leaked once, for `&'static`).
fn parse(table: &str) -> HashMap<String, &'static str> {
    table
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split_once('\t'))
        .map(|(en, tr)| (unescape(en), &*Box::leak(unescape(tr).into_boxed_str())))
        .collect()
}

fn tables() -> &'static HashMap<Language, HashMap<String, &'static str>> {
    static TABLES_PARSED: OnceLock<HashMap<Language, HashMap<String, &'static str>>> =
        OnceLock::new();
    TABLES_PARSED.get_or_init(|| TABLES.iter().map(|(l, _, t)| (*l, parse(t))).collect())
}

/// The translation of an English text in `language`, `None` when missing.
pub fn translate(language: Language, text: &str) -> Option<&'static str> {
    tables().get(&language.resolved())?.get(text).copied()
}

/// The French table, kept for callers and tests.
pub fn french(text: &str) -> Option<&'static str> {
    translate(Language::French, text)
}

static CURRENT: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Make [`tr`] follow `language` from now on.
pub fn set_language(language: Language) {
    CURRENT.store(language.resolved() as u8, Ordering::Relaxed);
}

/// The language [`tr`] currently uses (never `System`).
pub fn current() -> Language {
    let index = CURRENT.load(Ordering::Relaxed);
    Language::ALL
        .into_iter()
        .find(|l| *l as u8 == index)
        .unwrap_or(Language::English)
}

/// Is the interface currently in French?
pub fn is_french() -> bool {
    current() == Language::French
}

/// Translate an English interface text (identity in English or when missing).
pub fn tr(text: &'static str) -> &'static str {
    match current() {
        Language::English | Language::System => text,
        language => translate(language, text).unwrap_or(text),
    }
}

/// Marks a text for translation where `tr` cannot be called yet (constants and
/// tables): the call site then runs `tr` on the value.
pub const fn noop(text: &'static str) -> &'static str {
    text
}

/// Replace the `{}` placeholders of `template`, in order, with `args`.
pub fn fill(template: &str, args: &[&dyn Display]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut args = args.iter();
    let mut rest = template;
    while let Some(at) = rest.find("{}") {
        out.push_str(&rest[..at]);
        match args.next() {
            Some(arg) => out.push_str(&arg.to_string()),
            None => out.push_str("{}"),
        }
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    out
}

/// Display name of a favorite or friend group ("Favorites1" -> "Favoris1").
pub fn group_name(group: &str) -> String {
    for english in ["Favorites", "Friends"] {
        if let Some(number) = group.strip_prefix(english) {
            let shown = translate(current(), english).unwrap_or(english);
            return format!("{shown}{number}");
        }
    }
    group.to_string()
}

/// Is the operating system interface in French?
pub fn system_is_french() -> bool {
    system_language().is_some_and(|code| from_code(&code) == Some(Language::French))
}

#[cfg(windows)]
fn system_language() -> Option<String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    // SAFETY: plain query without arguments.
    let id = unsafe { GetUserDefaultUILanguage() };
    // Primary language ids, whatever the region.
    let code = match id & 0x3ff {
        0x0c => "fr",
        0x0a => "es",
        0x16 => "pt",
        0x10 => "it",
        0x07 => "de",
        0x19 => "ru",
        _ => "en",
    };
    Some(code.to_string())
}

#[cfg(not(windows))]
fn system_language() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty())
}
