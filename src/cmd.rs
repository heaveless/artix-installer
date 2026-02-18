use std::{
    fs::OpenOptions,
    io,
    process::{Command, Stdio},
};

use dialoguer::Confirm;

use crate::{error::InstallerError, ui};

// ── Internal helpers ──────────────────────────────────────────────────────────

fn not_found_or_io(program: &str, err: io::Error) -> InstallerError {
    if err.kind() == io::ErrorKind::NotFound {
        InstallerError::CommandNotFound(program.to_string())
    } else {
        InstallerError::Io(err)
    }
}

fn print_captured_output(stdout: &[u8], stderr: &[u8]) {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    if !out.trim().is_empty() {
        eprintln!("{}", out.trim());
    }
    if !err.trim().is_empty() {
        eprintln!("{}", err.trim());
    }
}

/// Maps a binary name to its Artix/Arch package name.
fn package_for(program: &str) -> &str {
    match program {
        "mkfs.fat" | "mkfs.vfat" | "fsck.fat" | "fatlabel" => "dosfstools",
        "mkfs.ext4" | "mkfs.ext3" | "mkfs.ext2" | "e2fsck" | "resize2fs" | "tune2fs" => {
            "e2fsprogs"
        }
        "mkfs.btrfs" | "btrfs" => "btrfs-progs",
        "mkfs.xfs" | "xfs_repair" => "xfsprogs",
        "mkswap" | "swapon" | "swapoff" | "mount" | "umount" | "cfdisk" | "fdisk"
        | "lsblk" | "blkid" | "findmnt" => "util-linux",
        "basestrap" | "fstabgen" | "artix-chroot" => "artools",
        "rc-service" | "rc-update" | "openrc" => "openrc",
        "ntpd" | "ntpdate" | "ntpq" => "ntp",
        other => other,
    }
}

/// When `program` is not found, asks the user if they want to install the
/// correct package via `pacman -S`. Returns `Ok(())` if installed successfully,
/// or `Err(CommandNotFound)` if the user declines.
fn offer_install(program: &str) -> Result<(), InstallerError> {
    let pkg = package_for(program);

    ui::print_warning(&format!("Command '{}' not found.", program));
    println!();

    if !Confirm::new()
        .with_prompt(&format!("Install '{}' with pacman?", pkg))
        .default(true)
        .interact()?
    {
        return Err(InstallerError::CommandNotFound(program.to_string()));
    }

    println!();
    let status = Command::new("pacman")
        .args(["-Sy", "--noconfirm", pkg])
        .status()
        .map_err(|e| not_found_or_io("pacman", e))?;

    if !status.success() {
        return Err(InstallerError::CommandFailed(
            "pacman".to_string(),
            status.code().unwrap_or(-1),
        ));
    }

    // Verify the binary is actually resolvable in PATH before returning.
    // This guarantees pacman's install is fully visible to the OS before
    // the caller retries the command.
    let available = Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !available {
        ui::print_error(&format!(
            "'{}' still not found after install. Check the package name.",
            program
        ));
        return Err(InstallerError::CommandNotFound(program.to_string()));
    }

    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Runs a command silently, discarding all output and ignoring any error.
/// Use for cleanup operations where partial failure is acceptable (e.g. umount).
pub fn run_best_effort(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Run a command that **takes over the terminal** (stdin/stdout/stderr inherited).
/// Use for interactive programs: `cfdisk`, `basestrap`, `artix-chroot`.
pub fn run_interactive(program: &str, args: &[&str]) -> Result<(), InstallerError> {
    let attempt = |p: &str, a: &[&str]| -> Result<(), InstallerError> {
        let status = Command::new(p)
            .args(a)
            .status()
            .map_err(|e| not_found_or_io(p, e))?;

        if !status.success() {
            return Err(InstallerError::CommandFailed(
                p.to_string(),
                status.code().unwrap_or(-1),
            ));
        }
        Ok(())
    };

    match attempt(program, args) {
        Err(InstallerError::CommandNotFound(_)) => {
            offer_install(program)?;
            attempt(program, args)
        }
        other => other,
    }
}

/// Run a command **silently** while displaying a spinner.
/// On success prints `done_msg` with a ✓.
/// On failure prints captured output and returns an error.
pub fn run_with_spinner(
    program: &str,
    args: &[&str],
    spin_msg: &str,
    done_msg: &str,
) -> Result<(), InstallerError> {
    let attempt = |p: &str, a: &[&str]| -> Result<(), InstallerError> {
        let pb = ui::spinner(spin_msg);
        let result = Command::new(p)
            .args(a)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| not_found_or_io(p, e));
        pb.finish_and_clear();

        match result {
            Err(e) => Err(e),
            Ok(output) if !output.status.success() => {
                print_captured_output(&output.stdout, &output.stderr);
                Err(InstallerError::CommandFailed(
                    p.to_string(),
                    output.status.code().unwrap_or(-1),
                ))
            }
            Ok(_) => {
                ui::print_success(done_msg);
                Ok(())
            }
        }
    };

    match attempt(program, args) {
        Err(InstallerError::CommandNotFound(_)) => {
            offer_install(program)?;
            attempt(program, args)
        }
        other => other,
    }
}

/// Run a command, capture its stdout, and return it as a `String`.
pub fn run_capture(program: &str, args: &[&str]) -> Result<String, InstallerError> {
    let attempt = |p: &str, a: &[&str]| -> Result<String, InstallerError> {
        let output = Command::new(p)
            .args(a)
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| not_found_or_io(p, e))?;

        if !output.status.success() {
            return Err(InstallerError::CommandFailed(
                p.to_string(),
                output.status.code().unwrap_or(-1),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    };

    match attempt(program, args) {
        Err(InstallerError::CommandNotFound(_)) => {
            offer_install(program)?;
            attempt(program, args)
        }
        other => other,
    }
}

/// Run a command and **append** its stdout to a file (equivalent to `>> path`).
pub fn run_append_to_file(
    program: &str,
    args: &[&str],
    file_path: &str,
) -> Result<(), InstallerError> {
    let attempt = |p: &str, a: &[&str]| -> Result<(), InstallerError> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;

        let status = Command::new(p)
            .args(a)
            .stdout(file)
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| not_found_or_io(p, e))?;

        if !status.success() {
            return Err(InstallerError::CommandFailed(
                p.to_string(),
                status.code().unwrap_or(-1),
            ));
        }
        Ok(())
    };

    match attempt(program, args) {
        Err(InstallerError::CommandNotFound(_)) => {
            offer_install(program)?;
            attempt(program, args)
        }
        other => other,
    }
}
