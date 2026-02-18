use console::style;
use dialoguer::Confirm;

use crate::{cmd, error::InstallerError, ui};

/// Optionally enters the newly installed system via `artix-chroot`.
pub fn run() -> Result<(), InstallerError> {
    println!();
    ui::print_kv_box(
        "Post-chroot checklist",
        &[
            ("hostname", "echo myhostname > /etc/hostname"),
            ("timezone", "ln -sf /usr/share/zoneinfo/…  /etc/localtime"),
            ("locale", "edit /etc/locale.gen  →  locale-gen"),
            ("password", "passwd"),
            ("bootloader", "grub-install  →  grub-mkconfig"),
            ("network", "pacman -S networkmanager  →  enable it"),
        ],
    );
    println!();

    println!(
        "  {}",
        style("Tip: type 'exit' or press Ctrl-D to leave the chroot.")
            .dim()
            .italic()
    );
    println!();

    if !Confirm::new()
        .with_prompt("Enter the new system with artix-chroot now?")
        .default(true)
        .interact()?
    {
        println!();
        ui::print_warning("Skipping chroot.");
        ui::print_info("Enter manually any time:  artix-chroot /mnt");
        return Ok(());
    }

    println!();
    ui::print_info("Entering chroot…");
    println!("{}", style("─".repeat(52)).dim());
    println!();

    // artix-chroot is fully interactive — hand over the terminal.
    cmd::run_interactive("artix-chroot", &["/mnt"])?;

    println!();
    println!("{}", style("─".repeat(52)).dim());
    ui::print_success("Exited chroot.");
    println!();
    ui::print_info("Unmount and reboot when you are ready:");
    ui::print_info("  umount -R /mnt && reboot");
    println!();

    Ok(())
}
