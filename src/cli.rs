use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "hypr-vogix",
    about = "Monochromatic screen overlay for Hyprland",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Apply a monochromatic theme (e.g., military, amber, cyber)
    #[arg(short, long, value_name = "NAME")]
    pub theme: Option<String>,

    /// Set overlay intensity (0.0 = no effect, 1.0 = full monochrome)
    #[arg(short, long, default_value = "1.0", requires = "theme", value_parser = parse_opacity)]
    pub opacity: f32,

    /// Set brightness (0.1 = very dark, 1.0 = default, 2.0 = max bright)
    #[arg(short, long, default_value = "1.0", requires = "theme", value_parser = parse_brightness)]
    pub brightness: f32,

    /// Set color saturation (0.0 = gray, 1.0 = default, 2.0 = vivid)
    #[arg(short, long, default_value = "1.0", requires = "theme", value_parser = parse_saturation)]
    pub saturation: f32,

    /// Invert lightness using algorithm: oklab, hsv
    #[arg(short, long, value_name = "ALGO", requires = "theme", value_parser = parse_invert)]
    pub invert: Option<String>,

    /// Restore last saved state; use --theme/--opacity/etc. as fallback if no state exists
    #[arg(long)]
    pub restore: bool,

    /// Turn off the current overlay
    #[arg(long)]
    pub off: bool,

    /// Show current overlay status
    #[arg(long)]
    pub status: bool,

    /// List available themes
    #[arg(short, long)]
    pub list: bool,
}

fn parse_invert(s: &str) -> Result<String, String> {
    match s {
        "oklab" | "okhsl" | "hsv" => Ok(s.to_string()),
        _ => Err(format!(
            "unknown algorithm '{s}', available: oklab, okhsl, hsv"
        )),
    }
}

fn parse_opacity(s: &str) -> Result<f32, String> {
    let val: f32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if !(0.0..=1.0).contains(&val) {
        return Err(format!("opacity must be between 0.0 and 1.0, got {val}"));
    }
    Ok(val)
}

fn parse_brightness(s: &str) -> Result<f32, String> {
    let val: f32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if !(0.1..=2.0).contains(&val) {
        return Err(format!("brightness must be between 0.1 and 2.0, got {val}"));
    }
    Ok(val)
}

fn parse_saturation(s: &str) -> Result<f32, String> {
    let val: f32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if !(0.0..=2.0).contains(&val) {
        return Err(format!("saturation must be between 0.0 and 2.0, got {val}"));
    }
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_theme() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "military"]).unwrap();
        assert_eq!(cli.theme.as_deref(), Some("military"));
    }

    #[test]
    fn parse_off() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--off"]).unwrap();
        assert!(cli.off);
    }

    #[test]
    fn parse_list() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--list"]).unwrap();
        assert!(cli.list);
    }

    #[test]
    fn parse_opacity_valid() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--opacity", "0.5"]).unwrap();
        assert!((cli.opacity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_opacity_default() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "x"]).unwrap();
        assert!((cli.opacity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_opacity_out_of_range() {
        assert!(Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--opacity", "1.5"]).is_err());
    }

    #[test]
    fn parse_opacity_invalid() {
        assert!(Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--opacity", "abc"]).is_err());
    }

    #[test]
    fn parse_brightness_valid() {
        let cli =
            Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--brightness", "0.5"]).unwrap();
        assert!((cli.brightness - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_brightness_default() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "x"]).unwrap();
        assert!((cli.brightness - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_brightness_out_of_range() {
        assert!(
            Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--brightness", "3.0"]).is_err()
        );
        assert!(
            Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--brightness", "0.05"]).is_err()
        );
    }

    #[test]
    fn parse_saturation_valid() {
        let cli =
            Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--saturation", "1.5"]).unwrap();
        assert!((cli.saturation - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_saturation_default() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--theme", "x"]).unwrap();
        assert!((cli.saturation - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_saturation_out_of_range() {
        assert!(
            Cli::try_parse_from(["hypr-vogix", "--theme", "x", "--saturation", "3.0"]).is_err()
        );
    }

    #[test]
    fn parse_restore() {
        let cli = Cli::try_parse_from(["hypr-vogix", "--restore"]).unwrap();
        assert!(cli.restore);
    }

    #[test]
    fn parse_restore_with_fallback() {
        let cli = Cli::try_parse_from([
            "hypr-vogix",
            "--restore",
            "--theme",
            "military",
            "--opacity",
            "0.7",
        ])
        .unwrap();
        assert!(cli.restore);
        assert_eq!(cli.theme.as_deref(), Some("military"));
        assert!((cli.opacity - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn no_args_is_error() {
        assert!(Cli::try_parse_from(["hypr-vogix"]).is_err());
    }
}
