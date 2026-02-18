use std::path::Path;

use crate::{error::InstallerError, ui};

/// Asserts that the system booted in UEFI mode.
/// Aborts with `BiosNotSupported` if not.
pub fn check() -> Result<(), InstallerError> {
    if Path::new("/sys/firmware/efi/efivars").exists() {
        ui::print_success("UEFI mode detected — EFI system partition required.");
        Ok(())
    } else {
        ui::print_error("BIOS/Legacy mode detected. This installer only supports UEFI.");
        Err(InstallerError::BiosNotSupported)
    }
}
