//! Workspace persistence. All file IO goes through a [`Workspace`] whose root
//! is injected at startup, so the same core runs standalone or jailed inside a
//! vault directory. Nothing here stores absolute paths.

use crate::bus::Value;
use crate::catalog;
use crate::model::RackState;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default standalone root: `$FORGE_HOME`, else the platform data dir.
    pub fn default_root() -> PathBuf {
        if let Some(p) = std::env::var_os("FORGE_HOME") {
            return PathBuf::from(p);
        }
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("biospark").join("forge")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self, name: &str) -> PathBuf {
        // Names are fixed identifiers; strip anything that could leave the root.
        let safe: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            .collect();
        self.root.join(format!("{}.json", safe.trim_matches('.')))
    }

    pub fn load<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let text = std::fs::read_to_string(self.path(name)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Write atomically (temp file + rename) so a crash never leaves half a file.
    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let path = self.path(name);
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, &path)
    }

    pub fn remove(&self, name: &str) {
        let _ = std::fs::remove_file(self.path(name));
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// "just now", "5m ago", "3h ago", "2d ago".
pub fn ago(then_ms: u64, now_ms: u64) -> String {
    let s = now_ms.saturating_sub(then_ms) as f64 / 1000.0;
    if s < 60.0 {
        "just now".into()
    } else if s < 3600.0 {
        format!("{}m ago", (s / 60.0).round())
    } else if s < 86400.0 {
        format!("{}h ago", (s / 3600.0).round())
    } else {
        format!("{}d ago", (s / 86400.0).round())
    }
}

/// Everything a vault version captures.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub rack: Option<RackState>,
    #[serde(default)]
    pub params: HashMap<String, Value>,
    /// Forge documents (panel tree, layers, layout), stored opaquely.
    #[serde(default)]
    pub forge: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub n: u32,
    pub at: u64,
    pub snap: Snapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub name: String,
    pub faction: String,
    pub created: u64,
    pub updated: u64,
    #[serde(default)]
    pub versions: Vec<Version>,
    /// File a vault was opened from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Working state saved when leaving the vault.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<Snapshot>,
    #[serde(default)]
    pub fresh: bool,
}

pub const MAX_VERSIONS: usize = 12;

impl Vault {
    pub fn new(name: &str, faction: &str) -> Self {
        let t = now_ms();
        let faction = if catalog::FACTIONS.iter().any(|f| f.0 == faction) {
            faction
        } else {
            "Venturan"
        };
        Self {
            id: format!("v{t:x}"),
            name: name.trim().to_uppercase(),
            faction: faction.into(),
            created: t,
            updated: t,
            versions: vec![],
            source: None,
            work: None,
            fresh: true,
        }
    }

    pub fn main() -> Self {
        let mut v = Self::new("BIOSPARK MAIN", "Venturan");
        v.id = "v-main".into();
        v.fresh = false;
        v
    }

    pub fn last_version(&self) -> Option<u32> {
        self.versions.last().map(|v| v.n)
    }

    /// Append a version, keeping the most recent [`MAX_VERSIONS`].
    pub fn save_version(&mut self, snap: Snapshot) -> u32 {
        let n = self.last_version().unwrap_or(0) + 1;
        let at = now_ms();
        self.versions.push(Version {
            n,
            at,
            snap: snap.clone(),
        });
        if self.versions.len() > MAX_VERSIONS {
            let cut = self.versions.len() - MAX_VERSIONS;
            self.versions.drain(..cut);
        }
        self.work = Some(snap);
        self.updated = at;
        n
    }
}

/// Load the vault list, creating the default vault on first run.
pub fn load_vaults(ws: &Workspace) -> Vec<Vault> {
    match ws.load::<Vec<Vault>>("vaults") {
        Some(v) if !v.is_empty() => v,
        _ => vec![Vault::main()],
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub user: String,
    /// `local` | `p2p` | `remote`.
    pub conn: String,
    #[serde(default)]
    pub host: String,
    pub vault: Option<String>,
}

/// Up to two initials, upper-cased ("Operator One" → "OO").
pub fn initials(user: &str) -> String {
    user.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_ws(tag: &str) -> Workspace {
        let dir =
            std::env::temp_dir().join(format!("forge-core-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Workspace::new(dir)
    }

    #[test]
    fn save_and_load_round_trip() {
        let ws = temp_ws("rt");
        let s = RackState::factory();
        ws.save("rack", &s).unwrap();
        assert_eq!(ws.load::<RackState>("rack"), Some(s));
        assert_eq!(ws.load::<RackState>("missing"), None);
        let _ = std::fs::remove_dir_all(ws.root());
    }

    #[test]
    fn names_cannot_escape_the_root() {
        let ws = temp_ws("esc");
        ws.save("../../evil", &1u8).unwrap();
        assert!(ws.root().join("evil.json").exists());
        let _ = std::fs::remove_dir_all(ws.root());
    }

    #[test]
    fn versions_are_capped() {
        let mut v = Vault::main();
        for _ in 0..20 {
            v.save_version(Snapshot::default());
        }
        assert_eq!(v.versions.len(), MAX_VERSIONS);
        assert_eq!(v.last_version(), Some(20));
        assert_eq!(v.versions[0].n, 9);
    }

    #[test]
    fn first_run_has_the_main_vault() {
        let ws = temp_ws("first");
        let v = load_vaults(&ws);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].name, "BIOSPARK MAIN");
    }

    #[test]
    fn helpers() {
        assert_eq!(initials("Operator One"), "OO");
        assert_eq!(initials("  jules "), "J");
        assert_eq!(ago(0, 30_000), "just now");
        assert_eq!(ago(0, 300_000), "5m ago");
        assert_eq!(ago(0, 7_200_000), "2h ago");
    }
}
