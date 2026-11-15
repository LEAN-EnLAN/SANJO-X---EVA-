//! A tiny `.env` reader (no crate): lets a teammate put `GROQ_API_KEY=…` in a
//! file next to the `eva` program instead of exporting it in a shell.
//!
//! Rules: one `KEY=VALUE` per line; blank lines and `#` comments are ignored;
//! a leading `export ` is allowed; spaces around key and value are trimmed;
//! one pair of matching `'…'` or `"…"` quotes is stripped; a UTF-8 BOM and
//! `\r` (Windows line ends) are removed; lines without `=` and keys that are not
//! `[A-Za-z_][A-Za-z0-9_]*` are skipped. **Variables already set in the process
//! environment are never overridden.**
//!
//! [`load`] must run before any other thread exists (`std::env::set_var` is not
//! thread-safe), which is why `main` is a plain `fn` that builds the tokio
//! runtime only after this.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Parse the text of an env file into `(key, value)` pairs, in file order.
pub fn parse(text: &str) -> Vec<(String, String)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = Vec::new();
    for raw in text.split('\n') {
        let line = raw.trim_end_matches('\r').trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim_start).unwrap_or(line);
        let Some((k, v)) = line.split_once('=') else { continue };
        let key = k.trim();
        if !valid_key(key) {
            continue;
        }
        let mut val = v.trim();
        if val.len() >= 2 {
            let (a, b) = (val.as_bytes()[0], val.as_bytes()[val.len() - 1]);
            if (a == b'"' && b == b'"') || (a == b'\'' && b == b'\'') {
                val = &val[1..val.len() - 1];
            }
        }
        out.push((key.to_string(), val.to_string()));
    }
    out
}

fn valid_key(k: &str) -> bool {
    let mut c = k.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_') && c.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Where the Groq key came from, for the startup log and `/api/eva/status` (never the key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Environment,
    DotEnv,
    EvaEnv,
    EnvFileFlag,
    None,
}

impl KeySource {
    pub fn as_str(self) -> &'static str {
        match self {
            KeySource::Environment => "environment",
            KeySource::DotEnv => ".env",
            KeySource::EvaEnv => "eva.env",
            KeySource::EnvFileFlag => "--env-file",
            KeySource::None => "none",
        }
    }
}

/// What [`load`] did.
#[derive(Debug, Clone)]
pub struct Report {
    /// the file that was read, if any
    pub file: Option<PathBuf>,
    /// every default place that was looked at (for the "no key" hint)
    pub looked: Vec<PathBuf>,
    pub key_source: KeySource,
    /// human words for the log: "eva.env next to the binary"
    pub where_words: String,
}

static KEY_SOURCE: OnceLock<KeySource> = OnceLock::new();

/// The key source recorded by [`load`] in this process (None if `load` never ran, e.g. in tests).
pub fn key_source() -> Option<KeySource> {
    KEY_SOURCE.get().copied()
}

/// The default search order: `./.env`, `<exe dir>/.env`, `<exe dir>/eva.env`.
pub fn default_candidates() -> Vec<(PathBuf, KeySource, String)> {
    let mut v = vec![(PathBuf::from(".env"), KeySource::DotEnv, ".env in the working directory".to_string())];
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        v.push((dir.join(".env"), KeySource::DotEnv, ".env next to the binary".into()));
        v.push((dir.join("eva.env"), KeySource::EvaEnv, "eva.env next to the binary".into()));
    }
    v
}

/// Load the env file: `explicit` (from `--env-file` / `EVA_ENV_FILE`) must exist;
/// otherwise the first default candidate that exists is read; none is fine.
/// Sets only variables that are not already in the environment.
///
/// Safety: call before spawning threads.
pub fn load(explicit: Option<&Path>) -> anyhow::Result<Report> {
    let key_in_env = std::env::var("GROQ_API_KEY").is_ok_and(|k| !k.trim().is_empty());
    let candidates = default_candidates();
    let looked: Vec<PathBuf> = candidates.iter().map(|c| c.0.clone()).collect();
    let chosen: Option<(PathBuf, KeySource, String)> = match explicit {
        Some(p) => {
            if !p.is_file() {
                anyhow::bail!("--env-file {}: no such file", p.display());
            }
            Some((p.to_path_buf(), KeySource::EnvFileFlag, format!("--env-file {}", p.display())))
        }
        None => candidates.into_iter().find(|c| c.0.is_file()),
    };
    let mut key_source = if key_in_env { KeySource::Environment } else { KeySource::None };
    let mut where_words = if key_in_env { "the environment".to_string() } else { String::new() };
    if let Some((path, src, words)) = &chosen {
        let text = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        for (k, v) in parse(&text) {
            if std::env::var_os(&k).is_some() {
                continue; // real environment variables win
            }
            // called from `main` before the tokio runtime (or any thread) starts
            std::env::set_var(&k, &v);
            if k == "GROQ_API_KEY" && !v.trim().is_empty() && !key_in_env {
                key_source = *src;
                where_words = words.clone();
            }
        }
    }
    let _ = KEY_SOURCE.set(key_source);
    Ok(Report { file: chosen.map(|c| c.0), looked, key_source, where_words })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv(t: &str) -> Vec<(String, String)> {
        parse(t)
    }

    #[test]
    fn plain_pairs_in_order() {
        assert_eq!(kv("A=1\nB=two"), vec![("A".into(), "1".into()), ("B".into(), "two".into())]);
    }

    #[test]
    fn blank_lines_and_comments_are_ignored() {
        assert_eq!(kv("\n# a comment\n   \n  # indented\nA=1\n"), vec![("A".into(), "1".into())]);
    }

    #[test]
    fn leading_export_is_allowed() {
        assert_eq!(kv("export GROQ_API_KEY=abc"), vec![("GROQ_API_KEY".into(), "abc".into())]);
    }

    #[test]
    fn spaces_are_trimmed() {
        assert_eq!(kv("  KEY  =   value with inner space   "), vec![("KEY".into(), "value with inner space".into())]);
    }

    #[test]
    fn one_pair_of_matching_quotes_is_stripped() {
        assert_eq!(kv("A=\"x y\"\nB='z'\nC=\"mismatch'\nD=\"\"x\"\""), vec![
            ("A".into(), "x y".into()),
            ("B".into(), "z".into()),
            ("C".into(), "\"mismatch'".into()),
            ("D".into(), "\"x\"".into()),
        ]);
    }

    #[test]
    fn bom_and_crlf_are_removed() {
        assert_eq!(kv("\u{feff}GROQ_API_KEY=k1\r\nB=2\r\n"), vec![("GROQ_API_KEY".into(), "k1".into()), ("B".into(), "2".into())]);
    }

    #[test]
    fn lines_without_equals_are_skipped() {
        assert_eq!(kv("just words\nA=1"), vec![("A".into(), "1".into())]);
    }

    #[test]
    fn invalid_keys_are_rejected() {
        assert_eq!(kv("1A=x\nA-B=y\nA B=z\n_OK9=1\n=empty"), vec![("_OK9".into(), "1".into())]);
    }

    #[test]
    fn values_may_contain_equals_and_hashes() {
        assert_eq!(kv("U=http://x/?a=b#frag"), vec![("U".into(), "http://x/?a=b#frag".into())]);
    }

    #[test]
    fn empty_value_is_kept() {
        assert_eq!(kv("GROQ_API_KEY="), vec![("GROQ_API_KEY".into(), "".into())]);
    }
}
