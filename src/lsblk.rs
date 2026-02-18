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

#[derive(Debug, Clone)]
pub struct Partition {
    pub path: String,      // /dev/sda1
    pub size: String,      // 512M
    pub part_type: String, // EFI System, Linux swap, Linux filesystem, …
}

impl Partition {
    /// One-line label shown in the arrow-key selector.
    pub fn display(&self) -> String {
        let type_label = if self.part_type.is_empty() {
            "(no type)"
        } else {
            &self.part_type
        };
        format!("{:<12}  {:>8}   {}", self.path, self.size, type_label)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns all block devices of type `disk` visible to the system.
/// Falls back to an empty list if `lsblk` is unavailable.
pub fn list_disks() -> Vec<Disk> {
    if crate::is_dry_run() {
        return mock_disks();
    }

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

/// Returns all partitions belonging to `disk` (e.g. `/dev/sda`).
/// Falls back to an empty list if `lsblk` is unavailable.
pub fn list_partitions(disk: &str) -> Vec<Partition> {
    if crate::is_dry_run() {
        return mock_partitions(disk);
    }

    let output = match cmd::run_capture(
        "lsblk",
        &["--pairs", "--output", "NAME,SIZE,TYPE,PARTTYPENAME", disk],
    ) {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    output
        .lines()
        .filter_map(|line| {
            let m = parse_pairs(line);
            if m.get("TYPE").map(String::as_str) != Some("part") {
                return None;
            }
            Some(Partition {
                path: format!("/dev/{}", m.get("NAME").map(String::as_str).unwrap_or("")),
                size: m.get("SIZE").cloned().unwrap_or_default(),
                part_type: m.get("PARTTYPENAME").cloned().unwrap_or_default(),
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
        // Find the '=' that separates key from value.
        let Some(eq) = rest.find('=') else { break };
        // The key is the last whitespace-delimited token before '='.
        let key = rest[..eq].split_whitespace().last().unwrap_or("").to_string();
        rest = &rest[eq + 1..];

        // Value is wrapped in double quotes.
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

// ── Mock data for dry-run (used on macOS / systems without lsblk) ─────────────

fn mock_disks() -> Vec<Disk> {
    vec![
        Disk {
            path: "/dev/sda".to_string(),
            size: "20G".to_string(),
            model: "QEMU HARDDISK".to_string(),
        },
        Disk {
            path: "/dev/sdb".to_string(),
            size: "8G".to_string(),
            model: "USB Flash Drive".to_string(),
        },
    ]
}

fn mock_partitions(disk: &str) -> Vec<Partition> {
    let base = disk.trim_start_matches("/dev/");
    vec![
        Partition {
            path: format!("/dev/{}1", base),
            size: "512M".to_string(),
            part_type: "EFI System".to_string(),
        },
        Partition {
            path: format!("/dev/{}2", base),
            size: "2G".to_string(),
            part_type: "Linux swap".to_string(),
        },
        Partition {
            path: format!("/dev/{}3", base),
            size: "17.5G".to_string(),
            part_type: "Linux filesystem".to_string(),
        },
    ]
}
