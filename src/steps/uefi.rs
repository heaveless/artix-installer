use std::path::Path;

use crate::{error::InstallerError, ui};

/// Detects whether the system booted in UEFI or BIOS/Legacy mode
/// by checking the presence of `/sys/firmware/efi/efivars`.
///
/// In dry-run mode the path won't exist on most dev machines (e.g. macOS),
/// so we simulate a UEFI result so the full flow can be exercised.
pub fn check() -> Result<bool, InstallerError> {
    let is_uefi = if crate::is_dry_run() {
        true // simulate UEFI so all prompts are exercised
    } else {
        Path::new("/sys/firmware/efi/efivars").exists()
    };

    if is_uefi {
        ui::print_success("UEFI mode detected — EFI system partition required.");
    } else {
        ui::print_warning("BIOS/Legacy mode detected — no EFI variables found.");
    }

    Ok(is_uefi)
}
