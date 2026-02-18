use console::style;
use dialoguer::{Confirm, Input, Select};

use crate::{cmd, error::InstallerError, lsblk, ui};

/// Shows available disks with arrow-key selection, then launches `cfdisk`.
/// Returns the chosen disk path (e.g. `/dev/sda`).
pub fn run() -> Result<String, InstallerError> {
    let disk = select_disk()?;

    let p1 = part_path(&disk.path, 1);
    let p2 = part_path(&disk.path, 2);
    let p3 = part_path(&disk.path, 3);
    let root_size = root_size_label(&disk.size);

    let efi_row  = format!("{} — EFI    1G",         p1);
    let boot_row = format!("{} — boot   1G",         p1);
    let swap_row = format!("{} — swap   10G",        p2);
    let root_row = format!("{} — root   {}", p3, root_size);

    println!();
    if is_uefi() {
        ui::print_kv_box(
            &format!("Suggested Layout — {} (UEFI)", disk.path),
            &[
                ("Part 1", efi_row.as_str()),
                ("Part 2", swap_row.as_str()),
                ("Part 3", root_row.as_str()),
            ],
        );
    } else {
        ui::print_kv_box(
            &format!("Suggested Layout — {} (BIOS)", disk.path),
            &[
                ("Part 1", boot_row.as_str()),
                ("Part 2", swap_row.as_str()),
                ("Part 3", root_row.as_str()),
            ],
        );
    }
    println!();
    ui::print_warning(&format!(
        "All data on {} will be erased when you write the partition table.",
        disk.path
    ));
    println!();

    if !Confirm::new()
        .with_prompt(&format!("Launch cfdisk on {}?", disk.path))
        .default(true)
        .interact()?
    {
        return Err(InstallerError::Cancelled);
    }

    // cfdisk is fully interactive — hand over the terminal.
    cmd::run_interactive("cfdisk", &[&disk.path])?;

    println!();
    ui::print_success("Partitioning complete. Returning to installer.");
    Ok(disk.path)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `/dev/<disk><n>` or `/dev/<disk>p<n>` for nvme/mmcblk devices.
pub fn part_path(disk: &str, n: u8) -> String {
    let base = disk.trim_start_matches("/dev/");
    if base.starts_with("nvme") || base.starts_with("mmcblk") {
        format!("/dev/{}p{}", base, n)
    } else {
        format!("/dev/{}{}", base, n)
    }
}

/// Returns `true` when the system booted in UEFI mode.
fn is_uefi() -> bool {
    std::path::Path::new("/sys/firmware/efi").exists()
}

/// Computes the leftover size after reserving 1 GiB (EFI/boot) + 10 GiB (swap).
fn root_size_label(total: &str) -> String {
    let bytes = parse_size_bytes(total);
    let used: u64 = 11 * 1024 * 1024 * 1024; // 11 GiB
    if bytes > used {
        format_gib(bytes - used)
    } else {
        "rest".to_string()
    }
}

/// Parses lsblk size strings like "20G", "931.5G", "1.8T", "512M" into bytes.
fn parse_size_bytes(s: &str) -> u64 {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('T') {
        (n.parse::<f64>().unwrap_or(0.0) * 1099511627776.0) as u64
    } else if let Some(n) = s.strip_suffix('G') {
        (n.parse::<f64>().unwrap_or(0.0) * 1073741824.0) as u64
    } else if let Some(n) = s.strip_suffix('M') {
        (n.parse::<f64>().unwrap_or(0.0) * 1048576.0) as u64
    } else {
        0
    }
}

fn format_gib(bytes: u64) -> String {
    let gib = bytes as f64 / 1073741824.0;
    if gib >= 1024.0 {
        format!("{:.1}T", gib / 1024.0)
    } else {
        format!("{:.0}G", gib)
    }
}

// ── Disk selection ────────────────────────────────────────────────────────────

fn select_disk() -> Result<lsblk::Disk, InstallerError> {
    let disks = lsblk::list_disks();

    if disks.is_empty() {
        // lsblk unavailable — fall back to manual input.
        ui::print_warning("Could not detect disks automatically.");
        let path: String = Input::new()
            .with_prompt("Enter disk path (e.g. /dev/sda)")
            .default("/dev/sda".to_string())
            .interact_text()?;
        return Ok(lsblk::Disk {
            path,
            size: "?".to_string(),
            model: "—".to_string(),
        });
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

    Ok(disks[idx].clone())
}
