use console::style;
use dialoguer::{Confirm, Input, Select};

use crate::{cmd, config::Config, error::InstallerError, lsblk, ui};

// ── Partition assignment ──────────────────────────────────────────────────────

/// Asks the user to assign roles to the partitions created in the previous step.
/// Uses arrow-key selection when partition info is available from `lsblk`.
pub fn ask_partitions(disk: &str, is_uefi: bool) -> Result<Config, InstallerError> {
    let partitions = lsblk::list_partitions(disk);

    if partitions.is_empty() {
        ui::print_warning("No partitions detected on the disk.");
        ui::print_warning("Make sure you wrote the partition table in cfdisk.");
    } else {
        ui::print_info(&format!(
            "Detected {} partition(s) on {}. Use ↑ ↓ to assign roles.",
            partitions.len(),
            disk
        ));
    }

    println!();

    // ── EFI / boot ────────────────────────────────────────────────────────────
    let efi_label = if is_uefi {
        "EFI partition  (→ FAT32, mounted at /boot)"
    } else {
        "Boot partition (→ FAT32, mounted at /boot)"
    };
    let efi_partition = select_partition(&partitions, efi_label, &format!("{}1", disk))?;

    // ── Swap (optional — last item in the list is "none") ─────────────────────
    println!();
    let swap_partition = select_partition_optional(
        &partitions,
        "Swap partition (→ mkswap)   [ select last item to skip ]",
    )?;

    // ── Root ──────────────────────────────────────────────────────────────────
    println!();
    let root_partition = select_partition(
        &partitions,
        "Root partition (→ ext4, mounted at /)",
        &format!("{}3", disk),
    )?;

    let config = Config {
        efi_partition,
        swap_partition,
        root_partition,
    };

    // ── Summary + confirmation ────────────────────────────────────────────────
    println!();
    let rows: Vec<(&str, String)> = vec![
        ("EFI/Boot", config.efi_partition.clone()),
        (
            "Swap",
            config
                .swap_partition
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
        ),
        ("Root", config.root_partition.clone()),
    ];
    ui::print_kv_box(
        "Partition Layout",
        &rows
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect::<Vec<_>>(),
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

/// Formats each partition according to its assigned role (with spinners).
pub fn run(config: &Config) -> Result<(), InstallerError> {
    cmd::run_with_spinner(
        "mkfs.fat",
        &["-F32", &config.efi_partition],
        &format!("Formatting {} as FAT32…", config.efi_partition),
        &format!("{} formatted as FAT32 (EFI/boot).", config.efi_partition),
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

// ── Selection helpers ─────────────────────────────────────────────────────────

/// Arrow-key selector for a required partition role.
/// Falls back to typed `Input` when no partition data is available.
fn select_partition(
    partitions: &[lsblk::Partition],
    prompt: &str,
    fallback_default: &str,
) -> Result<String, InstallerError> {
    if partitions.is_empty() {
        return Ok(Input::new()
            .with_prompt(prompt)
            .default(fallback_default.to_string())
            .interact_text()?);
    }

    print_partition_header();
    let labels: Vec<String> = partitions.iter().map(|p| p.display()).collect();

    let idx = Select::new()
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(partitions[idx].path.clone())
}

/// Arrow-key selector for an optional partition role.
/// Appends a "(none) — skip" entry at the bottom of the list.
fn select_partition_optional(
    partitions: &[lsblk::Partition],
    prompt: &str,
) -> Result<Option<String>, InstallerError> {
    if partitions.is_empty() {
        // No partition data — plain yes/no then typed input.
        if !Confirm::new()
            .with_prompt("Do you have a swap partition?")
            .default(true)
            .interact()?
        {
            return Ok(None);
        }
        let path: String = Input::new()
            .with_prompt("Swap partition")
            .default("/dev/sda2".to_string())
            .interact_text()?;
        return Ok(Some(path));
    }

    print_partition_header();

    let mut labels: Vec<String> = partitions.iter().map(|p| p.display()).collect();
    let none_idx = labels.len();
    labels.push(format!("{}", style("(none)  — no swap partition").dim().italic()));

    let idx = Select::new()
        .with_prompt(prompt)
        .items(&labels)
        .default(none_idx) // safest default: no swap
        .interact()?;

    if idx == none_idx {
        Ok(None)
    } else {
        Ok(Some(partitions[idx].path.clone()))
    }
}

/// Prints the column-header row above a partition selector.
fn print_partition_header() {
    println!(
        "  {:<12}  {:>8}   {}",
        style("PARTITION").dim(),
        style("SIZE").dim(),
        style("TYPE").dim()
    );
    println!("  {}", style("─".repeat(44)).dim());
}
