mod cmd;
mod config;
mod error;
mod lsblk;
mod session;
mod steps;
mod ui;

use dialoguer::Confirm;

use error::InstallerError;
use session::Session;

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    if let Err(e) = run() {
        println!();
        ui::print_error(&format!("{}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<(), InstallerError> {
    check_root()?;

    ui::print_banner();
    ui::print_info("This wizard will guide you through a full Artix Linux installation.");
    ui::print_info("You will be asked before each destructive operation.");

    let mut sess = check_resume()?;

    // ── Step 1: Detect boot mode ──────────────────────────────────────────────
    ui::print_step(1, 9, "System Mode Detection");
    if sess.last_step < 1 {
        steps::uefi::check()?;
        sess.last_step = 1;
        sess.save().ok();
    } else {
        ui::print_success("Already completed — skipping.");
    }

    // ── Step 2: Partition the disk ────────────────────────────────────────────
    ui::print_step(2, 9, "Disk Partitioning");
    let disk = if sess.last_step < 2 {
        let d = steps::partition::run()?;
        sess.disk = Some(d.clone());
        sess.last_step = 2;
        sess.save().ok();
        d
    } else {
        let d = sess.disk.clone().unwrap_or_default();
        ui::print_success(&format!("Already completed — disk: {}.", d));
        d
    };

    // ── Step 3: Assign roles + format ─────────────────────────────────────────
    ui::print_step(3, 9, "Partition Formatting");
    let config = if sess.last_step < 3 {
        let c = steps::format::build_config(&disk)?;
        steps::format::run(&c)?;
        sess.efi_partition  = Some(c.efi_partition.clone());
        sess.swap_partition = c.swap_partition.clone();
        sess.root_partition = Some(c.root_partition.clone());
        sess.last_step = 3;
        sess.save().ok();
        c
    } else {
        let c = sess.to_config();
        ui::print_success(&format!(
            "Already completed — EFI: {}  swap: {}  root: {}.",
            c.efi_partition,
            c.swap_partition.as_deref().unwrap_or("none"),
            c.root_partition,
        ));
        c
    };

    // ── Step 4: Mount the new filesystem ──────────────────────────────────────
    // Mounts are not persistent across process restarts, so always remount.
    ui::print_step(4, 9, "Mounting Partitions");
    steps::mount::run(&config)?;
    if sess.last_step < 4 {
        sess.last_step = 4;
        sess.save().ok();
    }

    // ── Step 5: Sync the system clock ─────────────────────────────────────────
    ui::print_step(5, 9, "Time Synchronization");
    if sess.last_step < 5 {
        steps::ntp::run()?;
        sess.last_step = 5;
        sess.save().ok();
    } else {
        ui::print_success("Already completed — skipping.");
    }

    // ── Step 6: Install base packages ─────────────────────────────────────────
    ui::print_step(6, 9, "Base System Installation");
    if sess.last_step < 6 {
        steps::packages::install_base()?;
        sess.last_step = 6;
        sess.save().ok();
    } else {
        ui::print_success("Already completed — skipping.");
    }

    // ── Step 7: Install kernel ────────────────────────────────────────────────
    ui::print_step(7, 9, "Kernel Installation");
    if sess.last_step < 7 {
        let kernel = steps::packages::ask_kernel()?;
        steps::packages::install_kernel(kernel)?;
        sess.kernel = Some(kernel);
        sess.last_step = 7;
        sess.save().ok();
    } else {
        ui::print_success(&format!(
            "Already completed — kernel: {}.",
            sess.kernel.map(|k| k.display_name()).unwrap_or("unknown"),
        ));
    }

    // ── Step 8: Desktop packages ──────────────────────────────────────────────
    ui::print_step(8, 9, "Desktop Environment");
    if sess.last_step < 8 {
        steps::packages::install_desktop()?;
        sess.last_step = 8;
        sess.save().ok();
    } else {
        ui::print_success("Already completed — skipping.");
    }

    // ── Step 9: Generate fstab + enter chroot ────────────────────────────────
    ui::print_step(9, 9, "Final Setup");
    steps::fstab::generate()?;
    steps::chroot::run()?;

    // Installation complete — remove checkpoint file.
    Session::clear();

    Ok(())
}

// ── Session resume prompt ─────────────────────────────────────────────────────

fn check_resume() -> Result<Session, InstallerError> {
    let Some(saved) = Session::load() else {
        return Ok(Session::default());
    };

    println!();
    ui::print_info(&format!(
        "Previous session found — completed step {}/9.",
        saved.last_step
    ));
    println!();

    if Confirm::new()
        .with_prompt("Resume from last checkpoint? (N = start from scratch)")
        .default(true)
        .interact()?
    {
        ui::print_success("Resuming previous session.");
        Ok(saved)
    } else {
        Session::clear();
        ui::print_info("Starting fresh.");
        Ok(Session::default())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn check_root() -> Result<(), InstallerError> {
    let uid = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u32>().ok())
        })
        .unwrap_or(1);

    if uid != 0 {
        return Err(InstallerError::NotRoot);
    }

    Ok(())
}
