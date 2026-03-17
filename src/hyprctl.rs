use std::path::Path;
use std::process::Command;

use crate::errors::{AppError, Result};

/// Check that Hyprland is running by verifying the instance signature env var.
pub fn check_environment() -> Result<()> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        return Err(AppError::HyprlandNotRunning);
    }
    Ok(())
}

/// Apply a screen shader from the given file path.
pub fn set_shader(shader_path: &Path) -> Result<()> {
    log::info!("Applying shader: {}", shader_path.display());
    run_hyprctl(&[
        "keyword",
        "decoration:screen_shader",
        &shader_path.to_string_lossy(),
    ])
}

/// Clear the active screen shader.
pub fn clear_shader() -> Result<()> {
    log::info!("Clearing screen shader");
    run_hyprctl(&["keyword", "decoration:screen_shader", "[[EMPTY]]"])
}

fn run_hyprctl(args: &[&str]) -> Result<()> {
    let output = Command::new("hyprctl").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::HyprctlNotFound
        } else {
            AppError::HyprctlFailed {
                code: -1,
                detail: e.to_string(),
            }
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.is_empty() {
            stdout.into_owned()
        } else {
            stderr.into_owned()
        };
        return Err(AppError::HyprctlFailed {
            code: output.status.code().unwrap_or(-1),
            detail,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn check_environment_no_signature() {
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
        let err = check_environment().unwrap_err();
        assert!(matches!(err, AppError::HyprlandNotRunning));
    }

    #[test]
    #[serial]
    fn check_environment_with_signature() {
        unsafe { std::env::set_var("HYPRLAND_INSTANCE_SIGNATURE", "test_instance") };
        assert!(check_environment().is_ok());
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
    }

    #[test]
    fn hyprctl_not_found_maps_error() {
        // Simulate the error mapping logic from run_hyprctl
        let err = Command::new("nonexistent-binary-xyz")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::HyprctlNotFound
                } else {
                    AppError::HyprctlFailed {
                        code: -1,
                        detail: e.to_string(),
                    }
                }
            })
            .unwrap_err();
        assert!(matches!(err, AppError::HyprctlNotFound));
    }
}
