use console::style;
use dialoguer::{Confirm, Input, Select};

use crate::{cmd, error::InstallerError, lsblk, ui};

/// Shows available disks with arrow-key selection, then launches `cfdisk`.
/// Returns the chosen disk path (e.g. `/dev/sda`).
pub fn run() -> Result<String, InstallerError> {
    let disk = select_disk()?;

    println!();
    ui::print_kv_box(
        "Suggested Layout",
        &[
            ("UEFI", "sda1 = EFI  512M  |  sda2 = swap  |  sda3 = root"),
            ("BIOS", "sda1 = boot 512M  |  sda2 = swap  |  sda3 = root"),
        ],
    );
    println!();
    ui::print_warning(&format!(
        "All data on {} will be erased when you write the partition table.",
        disk
    ));
    println!();

    if !Confirm::new()
        .with_prompt(&format!("Launch cfdisk on {}?", disk))
        .default(true)
        .interact()?
    {
        return Err(InstallerError::Cancelled);
    }

    // cfdisk is fully interactive — hand over the terminal.
    cmd::run_interactive("cfdisk", &[&disk])?;

    println!();
    ui::print_success("Partitioning complete. Returning to installer.");
    Ok(disk)
}

// ── Disk selection ────────────────────────────────────────────────────────────

fn select_disk() -> Result<String, InstallerError> {
    let disks = lsblk::list_disks();

    if disks.is_empty() {
        // lsblk unavailable — fall back to manual input.
        ui::print_warning("Could not detect disks automatically.");
        let path: String = Input::new()
            .with_prompt("Enter disk path (e.g. /dev/sda)")
            .default("/dev/sda".to_string())
            .interact_text()?;
        return Ok(path);
    }

    println!();
    ui::print_info("Use ↑ ↓ arrow keys to select the target disk, then press Enter.");
    println!(
        "  {:<12}  {:>8}   {}",
        style("DISK").dim(),
        style("SIZE").dim(),
        style("MODEL").dim()
    );
    println!("  {}", style("─".repeat(44)).dim());

    let labels: Vec<String> = disks.iter().map(|d| d.display()).collect();

    let idx = Select::new()
        .with_prompt("Target disk")
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(disks[idx].path.clone())
}
