use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("hyprctl not found in PATH -- is Hyprland installed?")]
    HyprctlNotFound,

    #[error("Hyprland is not running (HYPRLAND_INSTANCE_SIGNATURE not set)")]
    HyprlandNotRunning,

    #[error("hyprctl failed with exit code {code}: {detail}")]
    HyprctlFailed { code: i32, detail: String },

    #[error("unknown theme '{0}' -- use --list to see available themes")]
    UnknownTheme(String),

    #[error("failed to write shader to {path}: {source}")]
    ShaderWriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to remove shader {path}: {source}")]
    ShaderRemoveFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write state to {path}: {source}")]
    StateWriteFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to read state from {path}: {source}")]
    StateReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to serialize state: {0}")]
    StateSerialize(String),

    #[error("failed to deserialize state: {0}")]
    StateDeserialize(String),

    #[error("no runtime directory available (XDG_RUNTIME_DIR not set, /tmp fallback failed)")]
    NoRuntimeDir,
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let err = AppError::HyprctlNotFound;
        assert!(err.to_string().contains("hyprctl not found"));

        let err = AppError::HyprlandNotRunning;
        assert!(err.to_string().contains("HYPRLAND_INSTANCE_SIGNATURE"));

        let err = AppError::UnknownTheme("foo".into());
        assert!(err.to_string().contains("foo"));

        let err = AppError::HyprctlFailed {
            code: 1,
            detail: "bad".into(),
        };
        assert!(err.to_string().contains("exit code 1"));
    }
}
