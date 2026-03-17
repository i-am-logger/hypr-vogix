mod cli;
mod errors;
mod hyprctl;
mod shader;
mod state;
mod theme;

use anyhow::{Context, Result};
use clap::Parser;

use cli::Cli;
use errors::AppError;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if cli.list {
        print_themes();
        return Ok(());
    }

    if cli.status {
        print_status()?;
        return Ok(());
    }

    if cli.off {
        hyprctl::check_environment()?;
        hyprctl::clear_shader().context("failed to clear shader")?;
        shader::cleanup_shaders().context("failed to clean up shader files")?;
        state::clear().context("failed to clear state")?;
        return Ok(());
    }

    if cli.restore {
        hyprctl::check_environment()?;

        // Try saved state first, fall back to CLI args
        let resolved = match state::load().context("failed to read state")? {
            Some(saved) => {
                log::info!("Restoring saved state: '{}'", saved.theme);
                saved
            }
            None => {
                // No saved state — use CLI args as fallback defaults
                let name = cli
                    .theme
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("no saved state and no --theme provided"))?;
                log::info!("No saved state, applying fallback: '{name}'");
                state::State {
                    theme: name.to_string(),
                    opacity: cli.opacity,
                    brightness: cli.brightness,
                    saturation: cli.saturation,
                    invert: cli.invert.clone(),
                }
            }
        };

        let theme = theme::find_theme(&resolved.theme)
            .ok_or_else(|| AppError::UnknownTheme(resolved.theme.clone()))?;

        if let Err(e) = hyprctl::clear_shader() {
            log::warn!("Failed to clear previous shader: {e}");
        }
        shader::cleanup_shaders().context("failed to clean up old shader files")?;

        let shader_path = shader::write_shader(
            theme,
            resolved.opacity,
            resolved.brightness,
            resolved.saturation,
            resolved.invert.as_deref(),
        )
        .context("failed to write shader")?;
        hyprctl::set_shader(&shader_path).context("failed to apply shader")?;

        state::save(&resolved).context("failed to save state")?;
        log::info!("Applied '{}'", theme.name);
        return Ok(());
    }

    if let Some(ref name) = cli.theme {
        let theme = theme::find_theme(name).ok_or_else(|| AppError::UnknownTheme(name.clone()))?;

        hyprctl::check_environment()?;

        // Clear existing shader so Hyprland detects the change
        if let Err(e) = hyprctl::clear_shader() {
            log::warn!("Failed to clear previous shader: {e}");
        }
        shader::cleanup_shaders().context("failed to clean up old shader files")?;

        let shader_path = shader::write_shader(
            theme,
            cli.opacity,
            cli.brightness,
            cli.saturation,
            cli.invert.as_deref(),
        )
        .context("failed to write shader")?;
        hyprctl::set_shader(&shader_path).context("failed to apply shader")?;

        state::save(&state::State {
            theme: theme.name.to_string(),
            opacity: cli.opacity,
            brightness: cli.brightness,
            saturation: cli.saturation,
            invert: cli.invert.clone(),
        })
        .context("failed to save state")?;

        log::info!("Applied '{}'", theme.name);
        return Ok(());
    }

    Ok(())
}

fn print_themes() {
    println!("Available themes:\n");
    for theme in theme::builtin_themes() {
        let (lo, hi) = theme.wavelength_range;
        let wavelength = format!("{lo}-{hi} nm");
        println!(
            "  {:<12} {:<14} {}",
            theme.name, wavelength, theme.description
        );
    }
    println!();
    println!("Usage: hypr-vogix --theme <NAME> [--opacity] [--brightness] [--saturation]");
}

fn print_status() -> Result<()> {
    match state::load().context("failed to read state")? {
        Some(s) => {
            println!("Active overlay:\n");
            println!("  theme:      {}", s.theme);
            println!("  opacity:    {:.1}", s.opacity);
            println!("  brightness: {:.1}", s.brightness);
            println!("  saturation: {:.1}", s.saturation);
            if let Some(ref algo) = s.invert {
                println!("  invert:     {algo}");
            }
        }
        None => {
            println!("No active overlay.");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn run_list() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--list"]).unwrap();
        assert!(run(cli).is_ok());
    }

    #[test]
    fn run_unknown_theme() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "nonexistent"]).unwrap();
        let err = run(cli).unwrap_err();
        assert!(err.downcast_ref::<AppError>().is_some());
    }

    #[test]
    #[serial]
    fn run_off_without_hyprland() {
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
        let cli = Cli::try_parse_from(["hypr-vogix", "--off"]).unwrap();
        let err = run(cli).unwrap_err();
        assert!(err.downcast_ref::<AppError>().is_some());
    }

    #[test]
    #[serial]
    fn run_theme_without_hyprland() {
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "military"]).unwrap();
        let err = run(cli).unwrap_err();
        assert!(err.downcast_ref::<AppError>().is_some());
    }

    #[test]
    fn run_status() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--status"]).unwrap();
        assert!(run(cli).is_ok());
    }

    #[test]
    #[serial]
    fn run_restore_without_hyprland() {
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
        let cli = Cli::try_parse_from([
            "hypr-vogix",
            "--restore",
            "--theme",
            "military",
            "--opacity",
            "0.7",
        ])
        .unwrap();
        let err = run(cli).unwrap_err();
        assert!(err.downcast_ref::<AppError>().is_some());
    }

    #[test]
    #[serial]
    fn run_restore_no_state_no_theme_is_error() {
        unsafe { std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE") };
        // --restore without state file and without --theme should fail
        // (but the check_environment error comes first in this test)
        let cli = Cli::try_parse_from(["hypr-vogix", "--restore"]).unwrap();
        assert!(run(cli).is_err());
    }
}
