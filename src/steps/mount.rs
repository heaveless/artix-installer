use crate::{cmd, config::Config, error::InstallerError};

/// Mounts all partitions into the installation tree under `/mnt`.
///
/// Mount order:
///   1. Root  → /mnt
///   2. Swap  → swapon
///   3. Create /mnt/boot
///   4. EFI   → /mnt/boot
pub fn run(config: &Config) -> Result<(), InstallerError> {
    // 1. Root
    cmd::run_with_spinner(
        "mount",
        &[&config.root_partition, "/mnt"],
        &format!("Mounting {} → /mnt…", config.root_partition),
        &format!("{} mounted at /mnt.", config.root_partition),
    )?;

    // 2. Swap (optional) — deactivate first in case it's already active (resume).
    if let Some(ref swap) = config.swap_partition {
        cmd::run_best_effort("swapoff", &[swap]);
        cmd::run_with_spinner(
            "swapon",
            &[swap],
            &format!("Activating swap on {}…", swap),
            &format!("Swap on {} activated.", swap),
        )?;
    }

    // 3. Create the boot mount-point
    cmd::run_with_spinner(
        "mkdir",
        &["-p", "/mnt/boot"],
        "Creating /mnt/boot…",
        "Directory /mnt/boot created.",
    )?;

    // 4. EFI / boot
    cmd::run_with_spinner(
        "mount",
        &[&config.efi_partition, "/mnt/boot"],
        &format!("Mounting {} → /mnt/boot…", config.efi_partition),
        &format!("{} mounted at /mnt/boot.", config.efi_partition),
    )?;

    Ok(())
}
