use console::style;
use dialoguer::Confirm;

use crate::{cmd, config::Config, error::InstallerError, steps::partition::part_path, ui};

// ── Config builder ────────────────────────────────────────────────────────────

/// Derives partition roles from the disk path using the fixed layout:
///   p1 → EFI  (FAT32)
///   p2 → swap
///   p3 → root (ext4)
///
/// Shows a summary box and asks for confirmation before returning.
pub fn build_config(disk: &str) -> Result<Config, InstallerError> {
    let config = Config {
        efi_partition:  part_path(disk, 1),
        swap_partition: Some(part_path(disk, 2)),
        root_partition: part_path(disk, 3),
    };

    println!();
    ui::print_kv_box(
        "Partition Layout",
        &[
            ("EFI  (FAT32)", config.efi_partition.as_str()),
            ("Swap",         config.swap_partition.as_deref().unwrap()),
            ("Root (ext4)",  config.root_partition.as_str()),
        ],
    );
    println!();
    println!(
        "  {}",
        style("⚠  THIS WILL PERMANENTLY ERASE THE SELECTED PARTITIONS.")
            .red()
            .bold()
    );
    println!();

    if !Confirm::new()
        .with_prompt("Format these partitions?")
        .default(false)
        .interact()?
    {
        return Err(InstallerError::Cancelled);
    }

    Ok(config)
}

// ── Formatting ────────────────────────────────────────────────────────────────

/// Formats each partition: FAT32 (EFI), swap, ext4 (root).
pub fn run(config: &Config) -> Result<(), InstallerError> {
    // Unmount anything left from a previous run before formatting.
    // umount -R /mnt covers root + EFI (/mnt/boot) in one shot.
    cmd::run_best_effort("umount", &["-R", "/mnt"]);
    if let Some(ref swap) = config.swap_partition {
        cmd::run_best_effort("swapoff", &[swap]);
    }

    cmd::run_with_spinner(
        "mkfs.fat",
        &["-F32", &config.efi_partition],
        &format!("Formatting {} as FAT32…", config.efi_partition),
        &format!("{} formatted as FAT32 (EFI).", config.efi_partition),
    )?;

    if let Some(ref swap) = config.swap_partition {
        cmd::run_with_spinner(
            "mkswap",
            &[swap],
            &format!("Initialising swap on {}…", swap),
            &format!("{} initialised as swap.", swap),
        )?;
    }

    cmd::run_with_spinner(
        "mkfs.ext4",
        &[&config.root_partition],
        &format!("Formatting {} as ext4…", config.root_partition),
        &format!("{} formatted as ext4 (root).", config.root_partition),
    )?;

    Ok(())
}
