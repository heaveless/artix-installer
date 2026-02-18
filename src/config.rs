/// Holds all user-selected installation parameters collected throughout the process.
#[derive(Debug, Clone)]
pub struct Config {
    pub efi_partition: String,
    pub swap_partition: Option<String>,
    pub root_partition: String,
}

/// Which Linux kernel variant to install.
#[derive(Debug, Clone, Copy)]
pub enum KernelVariant {
    Stable,
    Lts,
    Zen,
}

impl KernelVariant {
    /// The `basestrap` package name for this variant.
    pub fn package_name(self) -> &'static str {
        match self {
            KernelVariant::Stable => "linux",
            KernelVariant::Lts => "linux-lts",
            KernelVariant::Zen => "linux-zen",
        }
    }

    /// Human-readable label shown to the user.
    pub fn display_name(self) -> &'static str {
        match self {
            KernelVariant::Stable => "Linux stable",
            KernelVariant::Lts => "Linux LTS (long-term support)",
            KernelVariant::Zen => "Linux Zen (performance-optimized)",
        }
    }
}
