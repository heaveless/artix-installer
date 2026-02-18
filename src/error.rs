use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command '{0}' failed with exit code {1}")]
    CommandFailed(String, i32),

    #[error("Command '{0}' not found — is it installed?")]
    CommandNotFound(String),

    #[error("Installation cancelled by user")]
    Cancelled,

    #[error("This installer must be run as root (sudo)")]
    NotRoot,

    #[error("BIOS/Legacy mode is not supported — this installer requires UEFI")]
    BiosNotSupported,

    #[error("Prompt error: {0}")]
    Prompt(#[from] dialoguer::Error),
}
