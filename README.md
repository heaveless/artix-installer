# Artix Linux Installer

Interactive CLI installer for [Artix Linux](https://artixlinux.org) (OpenRC edition), written in Rust.

Guides you step-by-step through a full base installation — asking before every destructive operation — and drops you into a `chroot` ready to finish configuration.

```
   ░█████╗░██████╗░████████╗██╗██╗░░██╗
   ██╔══██╗██╔══██╗╚══██╔══╝██║╚██╗██╔╝
   ███████║██████╔╝░░░██║░░░██║░╚███╔╝░
   ██╔══██║██╔══██╗░░░██║░░░██║░██╔██╗░
   ██║░░██║██║░░██║░░░██║░░░██║██╔╝░██╗
   ╚═╝░░╚═╝╚═╝░░╚═╝░░░╚═╝░░░╚═╝╚═╝░░╚═╝
```

---

## Installation steps covered

| # | Step | Commands used |
|---|------|---------------|
| 1 | UEFI / BIOS detection | `ls /sys/firmware/efi/efivars` |
| 2 | Disk partitioning | `cfdisk` |
| 3 | Partition formatting | `mkfs.fat`, `mkswap`, `mkfs.ext4` |
| 4 | Mounting | `mount`, `swapon`, `mkdir` |
| 5 | Clock sync | `rc-service ntpd start` |
| 6 | Base system | `basestrap … base base-devel openrc elogind-openrc` |
| 7 | Kernel (stable / lts / zen) | `basestrap … linux linux-firmware` |
| 8 | fstab + chroot | `fstabgen`, `artix-chroot` |

---

## Dependencies

### Runtime (must be present on the live ISO)

These are all included in the **Artix Linux base live environment** — no manual install needed when booting from the official ISO.

| Tool | Package | Purpose |
|------|---------|---------|
| `cfdisk` | `util-linux` | Interactive partition editor |
| `mkfs.fat` | `dosfstools` | Format EFI partition as FAT32 |
| `mkswap` / `swapon` | `util-linux` | Create and activate swap |
| `mkfs.ext4` | `e2fsprogs` | Format root partition as ext4 |
| `mount` / `mkdir` | `util-linux` / `coreutils` | Mount partitions |
| `rc-service` | `openrc` | Start the NTP daemon |
| `basestrap` | `artix-install-scripts` | Install packages into `/mnt` |
| `fstabgen` | `artix-install-scripts` | Generate `/mnt/etc/fstab` |
| `artix-chroot` | `artix-install-scripts` | Chroot into the new system |
| `lsblk` | `util-linux` | List block devices (informational) |
| `ntpd` | `ntp` | Network time synchronization |

### Build (only needed to compile the installer)

| Tool | Version | Purpose |
|------|---------|---------|
| [Rust](https://rustup.rs) | ≥ 1.75 | Compiler + Cargo |

Rust crate dependencies (managed automatically by Cargo):

| Crate | Version | Purpose |
|-------|---------|---------|
| [`dialoguer`](https://crates.io/crates/dialoguer) | 0.11 | Interactive prompts (select, confirm, input) |
| [`console`](https://crates.io/crates/console) | 0.15 | Terminal styling and colors |
| [`indicatif`](https://crates.io/crates/indicatif) | 0.17 | Animated spinners |
| [`thiserror`](https://crates.io/crates/thiserror) | 1 | Ergonomic error types |

---

## Usage

### On the live ISO (real installation)

```bash
# 1. Boot the Artix Linux live ISO
#    https://artixlinux.org/download.php

# 2. Download or copy the binary onto the live system, then run:
sudo ./artix-installer
```

The installer **must be run as root** — `mount`, `mkfs`, and `basestrap` all require it.

### Building from source

```bash
# Clone the repo
git clone https://github.com/youruser/artix-installer
cd artix-installer

# Build release binary
cargo build --release

# The binary will be at:
./target/release/artix-installer
```

To cross-compile from macOS to Linux x86_64:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
# Binary: target/x86_64-unknown-linux-musl/release/artix-installer
```

---

## Dry-run mode (local development / UI testing)

Test the full interactive flow **without root and without touching any disk**:

```bash
cargo run -- --dry-run
```

In dry-run mode:

- Root check is bypassed
- UEFI is simulated as detected
- Every command (`mkfs`, `mount`, `basestrap`…) is **replaced by a spinner + log line** showing what *would* have been executed:

```
  ⠙  Formatting /dev/sda1 as FAT32…
  [dry-run]  mkfs.fat -F32 /dev/sda1
  ✓  /dev/sda1 formatted as FAT32 (EFI/boot).
```

All prompts, menus, and confirmation screens are fully functional.

---

## Testing with a virtual machine (QEMU)

For end-to-end testing with real disk operations:

```bash
# 1. Download the Artix base ISO
#    https://artixlinux.org/download.php

# 2. Create a 20 GB virtual disk
qemu-img create -f qcow2 artix-test.qcow2 20G

# 3. Boot the ISO with the virtual disk attached
qemu-system-x86_64 \
  -m 2G \
  -enable-kvm \
  -cdrom artix-base-openrc-*.iso \
  -drive file=artix-test.qcow2,format=qcow2 \
  -boot d \
  -vga virtio

# 4. Inside the live environment, copy and run the installer
sudo ./artix-installer
```

> Remove `-enable-kvm` if you are on macOS or a system without KVM support.
> The VM will be slower but still fully functional.

---

## Post-chroot checklist

After the installer drops you into the chroot (`artix-chroot /mnt`), complete these steps manually:

```bash
# Hostname
echo myhostname > /etc/hostname

# Timezone
ln -sf /usr/share/zoneinfo/Region/City /etc/localtime
hwclock --systohc

# Locale
nano /etc/locale.gen          # uncomment your locale, e.g. en_US.UTF-8
locale-gen
echo LANG=en_US.UTF-8 > /etc/locale.conf

# Root password
passwd

# Bootloader (UEFI with GRUB)
pacman -S grub efibootmgr
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=artix
grub-mkconfig -o /boot/grub/grub.cfg

# Network manager
pacman -S networkmanager networkmanager-openrc
rc-update add NetworkManager default

# Exit chroot and reboot
exit
umount -R /mnt
reboot
```

---

## License

MIT
