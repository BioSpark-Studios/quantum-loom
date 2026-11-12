//! Genesis Container mounting (read side).
//!
//! A mounted Genesis is the "disc": Genesis → Mythos → Container → Capsule.
//! This module reads the plaintext `{header, manifest}` form. Sealed packages
//! mount header-only; unsealing (Argon2id → AES-256-GCM with the Seal file)
//! belongs to the shared `qgcp` crate and is not reimplemented here.

use serde::{Deserialize, Serialize};

/// The world Genesis that ships with the rack and mounts by default.
pub const OROS_V7: &str = include_str!("../assets/oros-v7.genesis.json");

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Header {
    pub format_version: String,
    pub package_id: String,
    pub name: String,
    pub domain: String,
    pub lifecycle: String,
    pub capsule_count: u32,
    pub capsule_capacity: u32,
    pub wire_types: Vec<String>,
    pub seal_id: String,
    pub description: String,
    pub depends_on: Vec<String>,
    pub cipher_registered: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Payload {
    pub description: String,
    pub index: u32,
    /// Length in scenes (16 beats = 1 scene).
    pub scenes: u32,
    pub intensity: f32,
    /// love | desire | fear | hope | wrath | grief
    pub mood: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Capsule {
    pub id: String,
    pub name: String,
    pub wire_type: String,
    pub payload: Payload,
    pub lineage_hash: String,
    pub resonance_hz: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ContainerMeta {
    pub pace_bpm: f32,
    pub mode: String,
    pub channel: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub lifecycle: String,
    pub metadata: ContainerMeta,
    pub capsules: Vec<Capsule>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mythos {
    pub id: String,
    pub name: String,
    pub crest: String,
    pub department: String,
    pub lifecycle: String,
    pub containers: Vec<Container>,
}

impl Mythos {
    pub fn capsule_count(&self) -> usize {
        self.containers.iter().map(|c| c.capsules.len()).sum()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub lifecycle: String,
    pub mythos: Vec<Mythos>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Genesis {
    pub header: Header,
    #[serde(default)]
    pub manifest: Option<Manifest>,
}

impl Genesis {
    /// Parse `{header, manifest}` JSON, or a bare manifest.
    pub fn parse(text: &str) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("not valid JSON: {e}"))?;
        if v.get("header").is_some() {
            return serde_json::from_value(v).map_err(|e| format!("not a Genesis package: {e}"));
        }
        if v.get("mythos").is_some() {
            let manifest: Manifest =
                serde_json::from_value(v).map_err(|e| format!("not a Genesis manifest: {e}"))?;
            let capsule_count = manifest
                .mythos
                .iter()
                .map(Mythos::capsule_count)
                .sum::<usize>() as u32;
            let header = Header {
                name: manifest.name.clone(),
                domain: manifest.domain.clone(),
                lifecycle: if manifest.lifecycle.is_empty() {
                    "active".into()
                } else {
                    manifest.lifecycle.clone()
                },
                capsule_count,
                capsule_capacity: 4096,
                ..Default::default()
            };
            return Ok(Self {
                header,
                manifest: Some(manifest),
            });
        }
        Err("no header or mythos found".into())
    }

    pub fn oros_v7() -> Self {
        Self::parse(OROS_V7).unwrap_or_default()
    }

    /// Sealed packages carry only their header.
    pub fn sealed(&self) -> bool {
        self.header.lifecycle == "sealed" && self.manifest.is_none()
    }

    pub fn mythos(&self) -> &[Mythos] {
        self.manifest
            .as_ref()
            .map(|m| m.mythos.as_slice())
            .unwrap_or(&[])
    }

    /// Stable signal colour for this package, derived from its id.
    pub fn channel(&self) -> &'static str {
        const CH: [&str; 6] = ["warm", "cool", "myth", "life", "rose", "amber"];
        let key = if self.header.package_id.is_empty() {
            &self.header.name
        } else {
            &self.header.package_id
        };
        let mut h: u32 = 7;
        for c in key.chars() {
            h = h.wrapping_mul(31).wrapping_add(c as u32);
        }
        CH[h as usize % CH.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_world_parses() {
        let g = Genesis::oros_v7();
        assert_eq!(g.header.name, "OROS-V7 · SYLLABARY");
        assert_eq!(g.mythos().len(), 16);
        assert_eq!(g.header.capsule_count, 256);
        let total: usize = g.mythos().iter().map(Mythos::capsule_count).sum();
        assert_eq!(total, 256);
        assert!(!g.sealed());
        assert_eq!(g.mythos()[0].crest, "atlas");
        assert!(!g.mythos()[0].containers[0].capsules[0]
            .payload
            .mood
            .is_empty());
    }

    #[test]
    fn sealed_header_only() {
        let g = Genesis::parse(r#"{"header":{"name":"ZETHYR-9","lifecycle":"sealed","capsuleCount":12,"capsuleCapacity":4096}}"#).unwrap();
        assert!(g.sealed());
        assert!(g.mythos().is_empty());
    }

    #[test]
    fn rejects_other_json() {
        assert!(Genesis::parse("{\"a\":1}").is_err());
        assert!(Genesis::parse("nope").is_err());
    }
}
