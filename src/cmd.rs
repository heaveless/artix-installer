use std::{
    fs::OpenOptions,
    io,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use console::style;

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

/// Prints what would have been executed (dry-run only).
fn print_dry(program: &str, args: &[&str]) {
    println!(
        "  {}  {} {}",
        style("[dry-run]").black().on_yellow().bold(),
        style(program).cyan(),
        style(args.join(" ")).dim()
    );
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run a command that **takes over the terminal** (stdin/stdout/stderr inherited).
/// Use for interactive programs: `cfdisk`, `basestrap`, `artix-chroot`.
pub fn run_interactive(program: &str, args: &[&str]) -> Result<(), InstallerError> {
    if crate::is_dry_run() {
        print_dry(program, args);
        thread::sleep(Duration::from_millis(600));
        return Ok(());
    }

    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| not_found_or_io(program, e))?;

    if !status.success() {
        return Err(InstallerError::CommandFailed(
            program.to_string(),
            status.code().unwrap_or(-1),
        ));
    }

    Ok(())
}

/// Run a command **silently** while displaying a spinner.
/// On success prints `done_msg` with a ✓.
/// On failure prints captured output and returns an error.
///
/// Use for quick, non-interactive commands: `mkfs.*`, `mount`, `swapon`, etc.
pub fn run_with_spinner(
    program: &str,
    args: &[&str],
    spin_msg: &str,
    done_msg: &str,
) -> Result<(), InstallerError> {
    if crate::is_dry_run() {
        let pb = ui::spinner(spin_msg);
        thread::sleep(Duration::from_millis(800));
        pb.finish_and_clear();
        print_dry(program, args);
        ui::print_success(done_msg);
        return Ok(());
    }

    let pb = ui::spinner(spin_msg);

    let result = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| not_found_or_io(program, e));

    pb.finish_and_clear();

    match result {
        Err(e) => Err(e),
        Ok(output) if !output.status.success() => {
            print_captured_output(&output.stdout, &output.stderr);
            Err(InstallerError::CommandFailed(
                program.to_string(),
                output.status.code().unwrap_or(-1),
            ))
        }
        Ok(_) => {
            ui::print_success(done_msg);
            Ok(())
        }
    }
}

/// Run a command, capture its stdout, and return it as a `String`.
pub fn run_capture(program: &str, args: &[&str]) -> Result<String, InstallerError> {
    // run_capture is only used for lsblk (informational) — fine to run even in dry-run.
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| not_found_or_io(program, e))?;

    if !output.status.success() {
        return Err(InstallerError::CommandFailed(
            program.to_string(),
            output.status.code().unwrap_or(-1),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run a command and **append** its stdout to a file (equivalent to `>> path`).
pub fn run_append_to_file(
    program: &str,
    args: &[&str],
    file_path: &str,
) -> Result<(), InstallerError> {
    if crate::is_dry_run() {
        print_dry(program, &[&args.join(" "), ">>", file_path]);
        thread::sleep(Duration::from_millis(500));
        return Ok(());
    }

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    let status = Command::new(program)
        .args(args)
        .stdout(file)
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| not_found_or_io(program, e))?;

    if !status.success() {
        return Err(InstallerError::CommandFailed(
            program.to_string(),
            status.code().unwrap_or(-1),
        ));
    }

    Ok(())
}
