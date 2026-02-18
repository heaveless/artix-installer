use crate::{cmd, error::InstallerError, ui};

/// Generates `/mnt/etc/fstab` using UUIDs via `fstabgen`.
///
/// Equivalent to: `fstabgen -U /mnt >> /mnt/etc/fstab`
pub fn generate() -> Result<(), InstallerError> {
    // basestrap creates /mnt/etc, but guard just in case.
    std::fs::create_dir_all("/mnt/etc")?;

    let pb = ui::spinner("Generating /mnt/etc/fstab (UUID-based)…");
    let result = cmd::run_append_to_file("fstabgen", &["-U", "/mnt"], "/mnt/etc/fstab");

    if result.is_ok() {
        ui::done_spinner(pb, "fstab written to /mnt/etc/fstab.");
    } else {
        pb.finish_and_clear();
    }

    result
}
