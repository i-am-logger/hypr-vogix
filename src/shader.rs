use std::fs;
use std::path::PathBuf;

use crate::errors::{FocusError, Result};
use crate::theme::{self, Theme};

const SHADER_TEMPLATE: &str = r#"#version 300 es
precision highp float;

in vec2 v_texcoord;
uniform sampler2D tex;
out vec4 fragColor;

// Hypr-vogix monochromatic shader
const vec3 themeColor = vec3({R}, {G}, {B});
const float intensity = {INTENSITY};
const float brightness = {BRIGHTNESS};

// OKLAB color space conversions (Björn Ottosson)
vec3 srgb_to_oklab(vec3 c) {
    vec3 lin = pow(c, vec3(2.2));
    float l = 0.4122214708 * lin.r + 0.5363325363 * lin.g + 0.0514459929 * lin.b;
    float m = 0.2119034982 * lin.r + 0.6806995451 * lin.g + 0.1073969566 * lin.b;
    float s = 0.0883024619 * lin.r + 0.2817188376 * lin.g + 0.6299787005 * lin.b;
    float l_ = pow(l, 1.0 / 3.0);
    float m_ = pow(m, 1.0 / 3.0);
    float s_ = pow(s, 1.0 / 3.0);
    return vec3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    );
}

vec3 oklab_to_srgb(vec3 c) {
    float l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    float m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    float s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;
    vec3 lin = vec3(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
       -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
       -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    );
    return pow(clamp(lin, 0.0, 1.0), vec3(1.0 / 2.2));
}

void main() {
    vec4 pixColor = texture(tex, v_texcoord);

{INVERT_BLOCK}
    // Normal monochromatic pipeline
    float luminance = dot(pixColor.rgb, vec3({LUMA_R}, {LUMA_G}, {LUMA_B}));
    vec3 mono = luminance * themeColor * brightness;
    vec3 result = mix(pixColor.rgb, mono, intensity);
    fragColor = vec4(result, pixColor.a);
}
"#;

/// Generate GLSL shader source for a theme with intensity, brightness, saturation, and invert.
#[must_use]
pub fn generate_shader(
    theme: &Theme,
    intensity: f32,
    brightness: f32,
    saturation: f32,
    invert: bool,
    invert_experimental: bool,
) -> String {
    let color = theme.color.with_saturation(saturation);
    let invert_block = if invert_experimental {
        // Experimental: add theme color to every pixel
        concat!(
            "    // HSV value inversion mapped to theme color\n",
            "    float mx = max(max(pixColor.r, pixColor.g), pixColor.b);\n",
            "    float inv = 1.0 - mx;\n",
            "    float invBright = (0.1 + inv * 0.9) * brightness;\n",
            "    vec3 themed = themeColor * invBright;\n",
            "    vec3 colored = (mx > 0.001) ? pixColor.rgb * (invBright / mx) : vec3(invBright);\n",
            "    fragColor = vec4(mix(colored, themed, intensity), pixColor.a);\n",
            "    return;\n",
            "    fragColor = vec4(pixColor.rgb, pixColor.a);\n",
            "    return;\n",
        )
    } else if invert {
        concat!(
            "    // Invert perceptual lightness in OKLAB with gamut mapping\n",
            "    vec3 lab = srgb_to_oklab(pixColor.rgb);\n",
            "    lab.x = 1.0 - lab.x;\n",
            "    // Binary search for max chroma that fits sRGB gamut\n",
            "    float lo = 0.0, hi = 1.0;\n",
            "    for (int i = 0; i < 12; i++) {\n",
            "        float mid = (lo + hi) * 0.5;\n",
            "        vec3 test_lab = vec3(lab.x, lab.y * mid, lab.z * mid);\n",
            "        vec3 rgb = oklab_to_srgb(test_lab);\n",
            "        if (rgb.r >= 0.0 && rgb.r <= 1.0 && rgb.g >= 0.0 && rgb.g <= 1.0 && rgb.b >= 0.0 && rgb.b <= 1.0)\n",
            "            lo = mid;\n",
            "        else\n",
            "            hi = mid;\n",
            "    }\n",
            "    lab.y *= lo;\n",
            "    lab.z *= lo;\n",
            "    pixColor.rgb = oklab_to_srgb(lab);\n",
        )
    } else {
        ""
    };
    SHADER_TEMPLATE
        .replace("{INVERT_BLOCK}", invert_block)
        .replace("{R}", &format!("{:.4}", color.r))
        .replace("{G}", &format!("{:.4}", color.g))
        .replace("{B}", &format!("{:.4}", color.b))
        .replace("{INTENSITY}", &format!("{:.4}", intensity.clamp(0.0, 1.0)))
        .replace(
            "{BRIGHTNESS}",
            &format!("{:.4}", brightness.clamp(0.1, 2.0)),
        )
        .replace("{LUMA_R}", &format!("{:.4}", theme::LUMA_R))
        .replace("{LUMA_G}", &format!("{:.4}", theme::LUMA_G))
        .replace("{LUMA_B}", &format!("{:.4}", theme::LUMA_B))
}

/// Return the directory for shader files.
/// Prefers `$XDG_RUNTIME_DIR/focus/`, falls back to `/tmp/focus/`.
pub fn shader_dir() -> Result<PathBuf> {
    let base = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));

    if !base.exists() {
        return Err(FocusError::NoRuntimeDir);
    }

    Ok(base.join("hypr-vogix"))
}

/// Write the shader to disk and return its path.
pub fn write_shader(
    theme: &Theme,
    intensity: f32,
    brightness: f32,
    saturation: f32,
    invert: bool,
    invert_experimental: bool,
) -> Result<PathBuf> {
    let dir = shader_dir()?;
    fs::create_dir_all(&dir).map_err(|e| FocusError::ShaderWriteFailed {
        path: dir.clone(),
        source: e,
    })?;

    let inv = if invert_experimental {
        "-ie"
    } else if invert {
        "-inv"
    } else {
        ""
    };
    let path = dir.join(format!(
        "hypr-vogix-{}-i{:.0}-b{:.0}-s{:.0}{inv}.glsl",
        theme.name,
        intensity * 100.0,
        brightness * 100.0,
        saturation * 100.0
    ));
    let source = generate_shader(
        theme,
        intensity,
        brightness,
        saturation,
        invert,
        invert_experimental,
    );

    fs::write(&path, source).map_err(|e| FocusError::ShaderWriteFailed {
        path: path.clone(),
        source: e,
    })?;

    log::info!("Wrote shader to {}", path.display());
    Ok(path)
}

/// Remove all focus shader files from the shader directory.
pub fn cleanup_shaders() -> Result<()> {
    let dir = shader_dir()?;
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&dir).map_err(|e| FocusError::ShaderRemoveFailed {
        path: dir.clone(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("hypr-vogix-") && n.ends_with(".glsl"))
        {
            fs::remove_file(&path).map_err(|e| FocusError::ShaderRemoveFailed {
                path: path.clone(),
                source: e,
            })?;
            log::debug!("Removed {}", path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, Theme};
    use serial_test::serial;

    fn test_theme() -> Theme {
        Theme {
            name: "test",
            description: "test theme",
            color: Color::new(0.0, 1.0, 0.0),
            wavelength_range: (530, 560),
        }
    }

    fn amber_theme() -> Theme {
        Theme {
            name: "amber",
            description: "test",
            color: Color::new(1.0, 0.71, 0.0),
            wavelength_range: (598, 608),
        }
    }

    #[test]
    fn generate_contains_version() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.starts_with("#version 300 es"));
    }

    #[test]
    fn generate_contains_theme_color() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.contains("vec3(0.0000, 1.0000, 0.0000)"));
    }

    #[test]
    fn generate_contains_intensity() {
        let src = generate_shader(&test_theme(), 0.8, 1.0, 1.0, false, false);
        assert!(src.contains("const float intensity = 0.8000;"));
    }

    #[test]
    fn generate_full_intensity() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.contains("const float intensity = 1.0000;"));
    }

    #[test]
    fn generate_zero_intensity() {
        let src = generate_shader(&test_theme(), 0.0, 1.0, 1.0, false, false);
        assert!(src.contains("const float intensity = 0.0000;"));
    }

    #[test]
    fn generate_clamps_intensity() {
        let src = generate_shader(&test_theme(), 2.0, 1.0, 1.0, false, false);
        assert!(src.contains("const float intensity = 1.0000;"));

        let src = generate_shader(&test_theme(), -0.5, 1.0, 1.0, false, false);
        assert!(src.contains("const float intensity = 0.0000;"));
    }

    #[test]
    fn generate_has_valid_glsl_structure() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.contains("void main()"));
        assert!(src.contains("fragColor ="));
        assert!(src.contains("texture(tex, v_texcoord)"));
        assert!(src.contains("luminance"));
    }

    #[test]
    fn generate_brightness_default() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.contains("const float brightness = 1.0000;"));
    }

    #[test]
    fn generate_brightness_dim() {
        let src = generate_shader(&test_theme(), 1.0, 0.5, 1.0, false, false);
        assert!(src.contains("const float brightness = 0.5000;"));
    }

    #[test]
    fn generate_brightness_boost() {
        let src = generate_shader(&test_theme(), 1.0, 1.8, 1.0, false, false);
        assert!(src.contains("const float brightness = 1.8000;"));
    }

    #[test]
    fn generate_saturation_desaturate() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 0.0, false, false);
        assert!(src.contains("vec3(0.7152, 0.7152, 0.7152)"));
    }

    #[test]
    fn generate_saturation_boost() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.5, false, false);
        assert!(src.contains("vec3(0.0000, 1.0000, 0.0000)"));
    }

    #[test]
    fn generate_invert_off() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(!src.contains("exp("));
    }

    #[test]
    fn generate_invert_on() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, true, false);
        assert!(src.contains("srgb_to_oklab"));
    }

    #[test]
    fn generate_invert_experimental() {
        let src = generate_shader(&test_theme(), 1.0, 1.0, 1.0, false, true);
        assert!(src.contains("HSV value inversion"));
    }

    #[test]
    fn generate_amber_color() {
        let src = generate_shader(&amber_theme(), 1.0, 1.0, 1.0, false, false);
        assert!(src.contains("vec3(1.0000, 0.7100, 0.0000)"));
    }

    #[test]
    #[serial]
    fn shader_dir_returns_path() {
        let dir = shader_dir().unwrap();
        assert!(dir.to_string_lossy().ends_with("/hypr-vogix"));
    }

    #[test]
    #[serial]
    fn write_and_cleanup_shaders() {
        let original_xdg = std::env::var("XDG_RUNTIME_DIR").ok();
        let tmp = std::env::temp_dir().join("hypr-vogix-test");
        std::fs::create_dir_all(&tmp).unwrap();
        unsafe { std::env::set_var("XDG_RUNTIME_DIR", &tmp) };

        // Write
        let path = write_shader(&test_theme(), 1.0, 1.0, 1.0, false, false).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("#version 300 es"));

        // Cleanup
        cleanup_shaders().unwrap();
        assert!(!path.exists());

        // Cleanup when no files exist is fine
        cleanup_shaders().unwrap();

        // Restore
        let _ = std::fs::remove_dir_all(&tmp);
        match original_xdg {
            Some(val) => unsafe { std::env::set_var("XDG_RUNTIME_DIR", val) },
            None => unsafe { std::env::remove_var("XDG_RUNTIME_DIR") },
        }
    }
}
