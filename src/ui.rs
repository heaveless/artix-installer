use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ── Terminal helpers ──────────────────────────────────────────────────────────

fn term_width() -> usize {
    Term::stdout().size().1.max(60) as usize
}

// ── Banner ────────────────────────────────────────────────────────────────────

pub fn print_banner() {
    let _ = Term::stdout().clear_screen();

    // ASCII-art title in block letters (fits in ~50 columns)
    let logo = [
        r"   ░█████╗░██████╗░████████╗██╗██╗░░██╗",
        r"   ██╔══██╗██╔══██╗╚══██╔══╝██║╚██╗██╔╝",
        r"   ███████║██████╔╝░░░██║░░░██║░╚███╔╝░",
        r"   ██╔══██║██╔══██╗░░░██║░░░██║░██╔██╗░",
        r"   ██║░░██║██║░░██║░░░██║░░░██║██╔╝░██╗",
        r"   ╚═╝░░╚═╝╚═╝░░╚═╝░░░╚═╝░░░╚═╝╚═╝░░╚═╝",
    ];

    println!();
    for line in &logo {
        println!("{}", style(line).cyan().bold());
    }
    println!();
    println!(
        "{}",
        style("   Linux Installer  ·  OpenRC Edition  ·  v0.1.0")
            .dim()
            .italic()
    );
    println!();
    println!("{}", style("─".repeat(term_width().min(52))).dim());
    println!();
}

// ── Step header ───────────────────────────────────────────────────────────────

/// Prints a visually distinct numbered step header.
pub fn print_step(step: u8, total: u8, title: &str) {
    println!();
    let tag = style(format!(" {}/{} ", step, total)).black().on_cyan().bold();
    let heading = style(format!("  {}", title)).white().bold();
    println!("{}{}", tag, heading);
    println!("{}", style("─".repeat(term_width().min(52))).dim());
}

// ── Feedback messages ─────────────────────────────────────────────────────────

/// Green ✓ — operation completed successfully.
pub fn print_success(msg: &str) {
    println!("  {}  {}", style("✓").green().bold(), style(msg).green());
}

/// Blue → — neutral info / progress note.
pub fn print_info(msg: &str) {
    println!("  {}  {}", style("→").blue().bold(), msg);
}

/// Yellow ⚠  — non-fatal notice.
pub fn print_warning(msg: &str) {
    println!("  {}  {}", style("⚠").yellow().bold(), style(msg).yellow());
}

/// Red ✗ — error (written to stderr).
pub fn print_error(msg: &str) {
    eprintln!("  {}  {}", style("✗").red().bold(), style(msg).red());
}

// ── Info box ──────────────────────────────────────────────────────────────────

/// Renders a bordered key→value box in the terminal.
///
/// ```text
/// ┌─ Partition Layout ────────────────┐
/// │  EFI/Boot    /dev/sda1            │
/// │  Swap        /dev/sda2            │
/// │  Root        /dev/sda3            │
/// └───────────────────────────────────┘
/// ```
pub fn print_kv_box(title: &str, rows: &[(&str, &str)]) {
    const BOX_INNER: usize = 38;

    let dashes = "─".repeat(BOX_INNER.saturating_sub(title.chars().count() + 2));
    println!(
        "  ┌─ {} {}┐",
        style(title).white().bold(),
        style(&dashes).dim()
    );

    for (key, val) in rows {
        println!(
            "  │  {:<13}{}",
            style(*key).dim(),
            style(*val).white().bold()
        );
    }

    println!("  └{}┘", style("─".repeat(BOX_INNER + 2)).dim());
}

// ── Spinner ───────────────────────────────────────────────────────────────────

/// Returns a running braille spinner.
/// Call `pb.finish_and_clear()` (or the `done_spinner` helper) when done.
pub fn spinner(msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan.bold}  {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.into());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Clears the spinner and prints a success message in its place.
pub fn done_spinner(pb: ProgressBar, msg: &str) {
    pb.finish_and_clear();
    print_success(msg);
}
