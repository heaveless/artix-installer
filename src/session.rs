use std::{fs, io::Write};

use crate::config::{Config, KernelVariant};

const SESSION_FILE: &str = "/tmp/artix-installer.session";

// ── Session state ─────────────────────────────────────────────────────────────

/// Persisted checkpoint data written after each successful step.
/// Format on disk: simple `key=value` lines.
#[derive(Debug, Default)]
pub struct Session {
    /// Index of the last fully completed step (0 = nothing done yet).
    pub last_step: u8,
    pub disk: Option<String>,
    pub efi_partition: Option<String>,
    pub swap_partition: Option<String>,
    pub root_partition: Option<String>,
    pub kernel: Option<KernelVariant>,
}

impl Session {
    // ── Persistence ───────────────────────────────────────────────────────────

    /// Returns `Some(session)` if a valid checkpoint file exists, else `None`.
    pub fn load() -> Option<Self> {
        let content = fs::read_to_string(SESSION_FILE).ok()?;
        let mut s = Session::default();

        for line in content.lines() {
            let mut parts = line.splitn(2, '=');
            let (key, val) = match (parts.next(), parts.next()) {
                (Some(k), Some(v)) => (k.trim(), v.trim().to_string()),
                _ => continue,
            };
            match key {
                "step"   => s.last_step     = val.parse().unwrap_or(0),
                "disk"   => s.disk          = Some(val),
                "efi"    => s.efi_partition  = Some(val),
                "swap"   => s.swap_partition = Some(val),
                "root"   => s.root_partition = Some(val),
                "kernel" => s.kernel = Some(KernelVariant::from_str(&val)),
                _ => {}
            }
        }

        if s.last_step == 0 { None } else { Some(s) }
    }

    /// Writes the current state to disk. Errors are silently ignored by callers.
    pub fn save(&self) -> std::io::Result<()> {
        let mut out = format!("step={}\n", self.last_step);
        if let Some(ref v) = self.disk          { out.push_str(&format!("disk={}\n",  v)); }
        if let Some(ref v) = self.efi_partition  { out.push_str(&format!("efi={}\n",   v)); }
        if let Some(ref v) = self.swap_partition { out.push_str(&format!("swap={}\n",  v)); }
        if let Some(ref v) = self.root_partition { out.push_str(&format!("root={}\n",  v)); }
        if let Some(k)     = self.kernel         { out.push_str(&format!("kernel={}\n", k.as_str())); }

        let mut f = fs::File::create(SESSION_FILE)?;
        f.write_all(out.as_bytes())
    }

    /// Removes the session file (called on successful completion or fresh start).
    pub fn clear() {
        let _ = fs::remove_file(SESSION_FILE);
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Reconstructs a `Config` from saved partition data.
    /// Panics only if called when session data is incomplete (programmer error).
    pub fn to_config(&self) -> Config {
        Config {
            efi_partition:  self.efi_partition.clone().unwrap_or_default(),
            swap_partition: self.swap_partition.clone(),
            root_partition: self.root_partition.clone().unwrap_or_default(),
        }
    }
}

// ── KernelVariant ↔ string ────────────────────────────────────────────────────

impl KernelVariant {
    fn as_str(self) -> &'static str {
        match self {
            KernelVariant::Stable => "stable",
            KernelVariant::Lts    => "lts",
            KernelVariant::Zen    => "zen",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "lts" => KernelVariant::Lts,
            "zen" => KernelVariant::Zen,
            _     => KernelVariant::Stable,
        }
    }
}
