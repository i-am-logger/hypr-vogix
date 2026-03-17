use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};

/// Persisted state of the current focus overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub theme: String,
    pub opacity: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub invert: Option<String>,
}

/// Return the state file path: `$XDG_STATE_HOME/focus/state.toml`
/// Falls back to `~/.local/state/focus/state.toml`.
#[must_use]
pub fn state_path() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".local/state")
        });
    base.join("hypr-vogix/state.toml")
}

/// Write the current state to disk.
pub fn save(state: &State) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::StateWriteFailed {
            path: path.clone(),
            source: e,
        })?;
    }
    let content =
        toml::to_string_pretty(state).map_err(|e| AppError::StateSerialize(e.to_string()))?;
    fs::write(&path, content).map_err(|e| AppError::StateWriteFailed {
        path: path.clone(),
        source: e,
    })?;
    log::debug!("State saved to {}", path.display());
    Ok(())
}

/// Load the current state from disk, if it exists.
pub fn load() -> Result<Option<State>> {
    let path = state_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|e| AppError::StateReadFailed {
        path: path.clone(),
        source: e,
    })?;
    let state: State =
        toml::from_str(&content).map_err(|e| AppError::StateDeserialize(e.to_string()))?;
    Ok(Some(state))
}

/// Remove the state file.
pub fn clear() -> Result<()> {
    let path = state_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppError::StateWriteFailed {
            path: path.clone(),
            source: e,
        })?;
        log::debug!("State removed: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn with_temp_state_home<F: FnOnce()>(f: F) {
        let original = std::env::var("XDG_STATE_HOME").ok();
        let tmp = std::env::temp_dir().join("hypr-vogix-state-test");
        std::fs::create_dir_all(&tmp).unwrap();
        unsafe { std::env::set_var("XDG_STATE_HOME", &tmp) };

        f();

        let _ = std::fs::remove_dir_all(&tmp);
        match original {
            Some(val) => unsafe { std::env::set_var("XDG_STATE_HOME", val) },
            None => unsafe { std::env::remove_var("XDG_STATE_HOME") },
        }
    }

    #[test]
    fn state_path_uses_xdg() {
        let path = state_path();
        assert!(path.to_string_lossy().ends_with("hypr-vogix/state.toml"));
    }

    #[test]
    #[serial]
    fn save_and_load() {
        with_temp_state_home(|| {
            let state = State {
                theme: "military".into(),
                opacity: 1.0,
                brightness: 0.8,
                saturation: 1.2,
                invert: None,
            };
            save(&state).unwrap();

            let loaded = load().unwrap().unwrap();
            assert_eq!(loaded.theme, "military");
            assert!((loaded.opacity - 1.0).abs() < f32::EPSILON);
            assert!((loaded.brightness - 0.8).abs() < f32::EPSILON);
            assert!((loaded.saturation - 1.2).abs() < f32::EPSILON);
        });
    }

    #[test]
    #[serial]
    fn save_and_load_with_invert() {
        with_temp_state_home(|| {
            let state = State {
                theme: "cyber".into(),
                opacity: 0.8,
                brightness: 1.0,
                saturation: 1.0,
                invert: Some("oklab".into()),
            };
            save(&state).unwrap();

            let loaded = load().unwrap().unwrap();
            assert_eq!(loaded.theme, "cyber");
            assert_eq!(loaded.invert, Some("oklab".into()));
        });
    }

    #[test]
    #[serial]
    fn load_returns_none_when_missing() {
        with_temp_state_home(|| {
            let result = load().unwrap();
            assert!(result.is_none());
        });
    }

    #[test]
    #[serial]
    fn save_and_clear() {
        with_temp_state_home(|| {
            let state = State {
                theme: "cyber".into(),
                opacity: 1.0,
                brightness: 1.0,
                saturation: 1.0,
                invert: None,
            };
            save(&state).unwrap();
            assert!(state_path().exists());

            clear().unwrap();
            assert!(!state_path().exists());

            // Clear when already gone is fine
            clear().unwrap();
        });
    }

    #[test]
    #[serial]
    fn state_toml_format() {
        with_temp_state_home(|| {
            let state = State {
                theme: "amber".into(),
                opacity: 1.0,
                brightness: 1.0,
                saturation: 1.0,
                invert: None,
            };
            save(&state).unwrap();

            let content = std::fs::read_to_string(state_path()).unwrap();
            assert!(content.contains("theme = \"amber\""));
            assert!(content.contains("opacity"));
            assert!(content.contains("brightness"));
            assert!(content.contains("saturation"));
        });
    }
}
