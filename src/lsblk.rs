use std::collections::HashMap;

use crate::cmd;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Disk {
    pub path: String,  // /dev/sda
    pub size: String,  // 20G
    pub model: String, // SAMSUNG SSD 870
}

impl Disk {
    /// One-line label shown in the arrow-key selector.
    pub fn display(&self) -> String {
        format!("{:<12}  {:>8}   {}", self.path, self.size, self.model)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns all block devices of type `disk` visible to the system.
/// Falls back to an empty list if `lsblk` is unavailable.
pub fn list_disks() -> Vec<Disk> {
    let output = match cmd::run_capture(
        "lsblk",
        &["--pairs", "--output", "NAME,SIZE,TYPE,MODEL", "--nodeps"],
    ) {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    output
        .lines()
        .filter_map(|line| {
            let m = parse_pairs(line);
            if m.get("TYPE").map(String::as_str) != Some("disk") {
                return None;
            }
            Some(Disk {
                path: format!("/dev/{}", m.get("NAME").map(String::as_str).unwrap_or("")),
                size: m.get("SIZE").cloned().unwrap_or_default(),
                model: {
                    let s = m.get("MODEL").cloned().unwrap_or_default();
                    if s.is_empty() { "—".to_string() } else { s }
                },
            })
        })
        .collect()
}

// ── lsblk --pairs parser ──────────────────────────────────────────────────────
//
// Each line looks like:   NAME="sda1" SIZE="512M" TYPE="part" PARTTYPENAME="EFI System"

fn parse_pairs(line: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut rest = line.trim();

    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].split_whitespace().last().unwrap_or("").to_string();
        rest = &rest[eq + 1..];

        if !rest.starts_with('"') {
            break;
        }
        rest = &rest[1..]; // skip opening "

        let Some(close) = rest.find('"') else { break };
        let value = rest[..close].to_string();
        rest = &rest[close + 1..]; // skip closing "

        if !key.is_empty() {
            map.insert(key, value);
        }
    }

    map
}
