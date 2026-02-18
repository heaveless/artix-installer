use console::style;
use dialoguer::{Confirm, Select};

use crate::{cmd, config::KernelVariant, error::InstallerError, ui};

// ── Base system ───────────────────────────────────────────────────────────────

/// Installs the base Artix packages via `basestrap`.
pub fn install_base() -> Result<(), InstallerError> {
    ui::print_kv_box(
        "Packages to install",
        &[
            ("base", "core system utilities"),
            ("base-devel", "build tools (gcc, make, …)"),
            ("openrc", "init system"),
            ("elogind-openrc", "session management"),
        ],
    );
    println!();

    if !Confirm::new()
        .with_prompt("Proceed with base installation?")
        .default(true)
        .interact()?
    {
        return Err(InstallerError::Cancelled);
    }

    println!();
    // basestrap shows download progress — keep it interactive.
    cmd::run_interactive(
        "basestrap",
        &["/mnt", "base", "base-devel", "openrc", "elogind-openrc"],
    )?;

    ui::print_success("Base system installed.");
    Ok(())
}

// ── Kernel ────────────────────────────────────────────────────────────────────

/// Asks the user which kernel variant they want, then installs it.
pub fn ask_kernel() -> Result<KernelVariant, InstallerError> {
    println!();

    // Brief description of each variant shown before the prompt.
    ui::print_kv_box(
        "Kernel variants",
        &[
            ("stable", "latest mainline kernel — best hardware support"),
            ("lts", "long-term support — stability over features"),
            ("zen", "performance-tuned, lower latency — gaming/desktop"),
        ],
    );
    println!();

    let options = [
        format!("{}  {}", style("linux    ").cyan().bold(), style("stable (recommended)").dim()),
        format!("{}  {}", style("linux-lts").cyan().bold(), style("long-term support").dim()),
        format!("{}  {}", style("linux-zen").cyan().bold(), style("performance-optimized").dim()),
    ];

    let selection = Select::new()
        .with_prompt("Which kernel do you want to install?")
        .default(0)
        .items(&options)
        .interact()?;

    let kernel = match selection {
        0 => KernelVariant::Stable,
        1 => KernelVariant::Lts,
        2 => KernelVariant::Zen,
        _ => unreachable!(),
    };

    ui::print_info(&format!("Selected: {}", kernel.display_name()));
    Ok(kernel)
}

/// Installs the chosen kernel + `linux-firmware` via `basestrap`.
pub fn install_kernel(kernel: KernelVariant) -> Result<(), InstallerError> {
    let pkg = kernel.package_name();

    ui::print_info(&format!(
        "Installing {} + linux-firmware…",
        style(pkg).cyan().bold()
    ));
    println!();

    // basestrap streams download output — keep it interactive.
    cmd::run_interactive("basestrap", &["/mnt", pkg, "linux-firmware"])?;

    ui::print_success(&format!("Kernel '{}' installed.", pkg));
    Ok(())
}
