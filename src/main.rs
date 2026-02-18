mod cmd;
mod config;
mod error;
mod lsblk;
mod steps;
mod ui;

use std::sync::atomic::{AtomicBool, Ordering};

use error::InstallerError;

// ── Global dry-run flag ───────────────────────────────────────────────────────

/// When `true`, no system command is actually executed.
/// All shell operations are simulated with a short delay.
/// Set by passing `--dry-run` on the command line.
pub static DRY_RUN: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn is_dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed)
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    // Parse the only supported flag before doing anything else.
    if std::env::args().any(|a| a == "--dry-run") {
        DRY_RUN.store(true, Ordering::Relaxed);
    }

    if let Err(e) = run() {
        println!();
        ui::print_error(&format!("{}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<(), InstallerError> {
    // ── Guard ─────────────────────────────────────────────────────────────────
    check_root()?;

    // ── Welcome ───────────────────────────────────────────────────────────────
    ui::print_banner();

    if is_dry_run() {
        ui::print_warning("DRY-RUN MODE — no disk will be touched, no command will run.");
    }

    ui::print_info("This wizard will guide you through a full Artix Linux installation.");
    ui::print_info("You will be asked before each destructive operation.");

    // ── Step 1: Detect boot mode ──────────────────────────────────────────────
    ui::print_step(1, 8, "System Mode Detection");
    let is_uefi = steps::uefi::check()?;

    // ── Step 2: Partition the disk ────────────────────────────────────────────
    ui::print_step(2, 8, "Disk Partitioning");
    let disk = steps::partition::run()?;

    // ── Step 3: Choose partition roles + format ───────────────────────────────
    ui::print_step(3, 8, "Partition Formatting");
    let config = steps::format::ask_partitions(&disk, is_uefi)?;
    steps::format::run(&config)?;

    // ── Step 4: Mount the new filesystem ─────────────────────────────────────
    ui::print_step(4, 8, "Mounting Partitions");
    steps::mount::run(&config)?;

    // ── Step 5: Sync the system clock ─────────────────────────────────────────
    ui::print_step(5, 8, "Time Synchronization");
    steps::ntp::run()?;

    // ── Step 6: Install base packages ─────────────────────────────────────────
    ui::print_step(6, 8, "Base System Installation");
    steps::packages::install_base()?;

    // ── Step 7: Install kernel ────────────────────────────────────────────────
    ui::print_step(7, 8, "Kernel Installation");
    let kernel = steps::packages::ask_kernel()?;
    steps::packages::install_kernel(kernel)?;

    // ── Step 8: Generate fstab + enter chroot ────────────────────────────────
    ui::print_step(8, 8, "Final Setup");
    steps::fstab::generate()?;
    steps::chroot::run()?;

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Checks that the process is running as root (UID 0).
/// Skipped automatically in dry-run mode.
fn check_root() -> Result<(), InstallerError> {
    if is_dry_run() {
        return Ok(()); // no root needed to simulate
    }

    let uid = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u32>().ok())
        })
        .unwrap_or(1); // default to non-root if unreadable

    if uid != 0 {
        return Err(InstallerError::NotRoot);
    }

    Ok(())
}
