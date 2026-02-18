use dialoguer::Confirm;

use crate::{cmd, error::InstallerError, ui};

/// Optionally syncs the system clock via the NTP daemon.
/// An incorrect clock can cause package-signature validation to fail.
pub fn run() -> Result<(), InstallerError> {
    ui::print_info("An accurate clock prevents package-signature validation errors.");
    println!();

    if !Confirm::new()
        .with_prompt("Sync system time via NTP? (recommended)")
        .default(true)
        .interact()?
    {
        ui::print_warning("Skipping time synchronization — beware of signature issues.");
        return Ok(());
    }

    // -g: allow large time corrections  -q: one-shot, exit after sync
    cmd::run_with_spinner(
        "ntpd",
        &["-gq"],
        "Syncing system clock…",
        "System clock synchronized.",
    )?;

    Ok(())
}
