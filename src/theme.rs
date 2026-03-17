/// Rec. 709 luma coefficients for luminance calculation.
pub const LUMA_R: f32 = 0.2126;
pub const LUMA_G: f32 = 0.7152;
pub const LUMA_B: f32 = 0.0722;

/// Normalized RGB color with components in [0.0, 1.0].
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Adjust saturation. < 1.0 desaturates (toward gray),
    /// > 1.0 boosts (more vivid). 1.0 returns unchanged.
    #[must_use]
    pub fn with_saturation(&self, saturation: f32) -> Self {
        let gray = self.r * LUMA_R + self.g * LUMA_G + self.b * LUMA_B;
        Self {
            r: (gray + (self.r - gray) * saturation).clamp(0.0, 1.0),
            g: (gray + (self.g - gray) * saturation).clamp(0.0, 1.0),
            b: (gray + (self.b - gray) * saturation).clamp(0.0, 1.0),
        }
    }
}

/// A named monochromatic theme.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub color: Color,
    /// Dominant wavelength range in nm.
    pub wavelength_range: (u16, u16),
}

static THEMES: &[Theme] = &[
    Theme {
        name: "military",
        description: "Night-vision tactical green",
        color: Color::new(0.2, 1.0, 0.4), // P39 phosphor #33FF66
        wavelength_range: (530, 560),
    },
    Theme {
        name: "green",
        description: "Classic P1 green CRT",
        color: Color::new(0.29, 1.0, 0.0), // P1 phosphor 525nm #4AFF00
        wavelength_range: (520, 530),
    },
    Theme {
        name: "amber",
        description: "Classic amber CRT",
        color: Color::new(1.0, 0.71, 0.0), // P3 phosphor 602nm #FFB400
        wavelength_range: (598, 608),
    },
    Theme {
        name: "alert",
        description: "Red warning lights",
        color: Color::new(1.0, 0.0, 0.0), // Pure red
        wavelength_range: (620, 680),
    },
    Theme {
        name: "cyber",
        description: "Neon futuristic cyan",
        color: Color::new(0.0, 1.0, 1.0), // Pure cyan #00FFFF
        wavelength_range: (485, 500),
    },
    Theme {
        name: "arctic",
        description: "Cold ice blue",
        color: Color::new(0.0, 0.7, 1.0), // Vivid azure
        wavelength_range: (460, 480),
    },
    Theme {
        name: "cobalt",
        description: "Deep industrial blue",
        color: Color::new(0.0, 0.42, 1.0), // Cobalt #0047AB normalized
        wavelength_range: (450, 470),
    },
    Theme {
        name: "void",
        description: "Deep cosmic purple",
        color: Color::new(0.5, 0.0, 1.0), // Deep violet
        wavelength_range: (400, 430),
    },
    Theme {
        name: "toxic",
        description: "Radioactive yellow-green",
        color: Color::new(0.44, 1.0, 0.19), // Biohazard #61DE2A normalized
        wavelength_range: (550, 570),
    },
    Theme {
        name: "infrared",
        description: "Thermal camera magenta",
        color: Color::new(1.0, 0.0, 1.0), // Pure magenta #FF00FF
        wavelength_range: (620, 700),
    },
    Theme {
        name: "rose",
        description: "Soft lo-fi pink",
        color: Color::new(1.0, 0.4, 0.8), // Lo-fi pink #FF66CC
        wavelength_range: (600, 650),
    },
    Theme {
        name: "sepia",
        description: "Old photograph warmth",
        color: Color::new(1.0, 0.59, 0.18), // #704214 normalized for tint
        wavelength_range: (580, 620),
    },
    Theme {
        name: "walnut",
        description: "Dark stained wood",
        color: Color::new(0.7, 0.35, 0.1), // Dark warm brown
        wavelength_range: (580, 610),
    },
    Theme {
        name: "white",
        description: "Classic P4 white CRT",
        color: Color::new(1.0, 1.0, 1.0),
        wavelength_range: (380, 700),
    },
];

/// Return all built-in themes.
#[must_use]
pub fn builtin_themes() -> &'static [Theme] {
    THEMES
}

/// Look up a theme by name (case-insensitive).
#[must_use]
pub fn find_theme(name: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    // OKLAB reference implementation for validating shader inversion logic.
    // Used by contract tests to verify color transformations.
    impl Color {
        #[allow(clippy::excessive_precision)]
        fn to_oklab(&self) -> [f32; 3] {
            let r = self.r.powf(2.2);
            let g = self.g.powf(2.2);
            let b = self.b.powf(2.2);
            let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
            let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
            let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
            let l_ = l.cbrt();
            let m_ = m.cbrt();
            let s_ = s.cbrt();
            [
                0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
                1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
                0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
            ]
        }

        #[allow(clippy::excessive_precision)]
        fn from_oklab(lab: [f32; 3]) -> Self {
            let l_ = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
            let m_ = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
            let s_ = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];
            let l = l_ * l_ * l_;
            let m = m_ * m_ * m_;
            let s = s_ * s_ * s_;
            let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
            let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
            let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
            Self {
                r: r.clamp(0.0, 1.0).powf(1.0 / 2.2),
                g: g.clamp(0.0, 1.0).powf(1.0 / 2.2),
                b: b.clamp(0.0, 1.0).powf(1.0 / 2.2),
            }
        }

        #[allow(clippy::excessive_precision)]
        fn oklab_in_gamut(lab: [f32; 3]) -> bool {
            let l_ = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
            let m_ = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
            let s_ = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];
            let l = l_ * l_ * l_;
            let m = m_ * m_ * m_;
            let s = s_ * s_ * s_;
            let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
            let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
            let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
            (-0.001..=1.001).contains(&r)
                && (-0.001..=1.001).contains(&g)
                && (-0.001..=1.001).contains(&b)
        }

        fn invert_lightness(&self) -> Self {
            let mut lab = self.to_oklab();
            lab[0] = 1.0 - lab[0];
            let mut lo: f32 = 0.0;
            let mut hi: f32 = 1.0;
            for _ in 0..16 {
                let mid = (lo + hi) * 0.5;
                let test = [lab[0], lab[1] * mid, lab[2] * mid];
                if Self::oklab_in_gamut(test) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            lab[1] *= lo;
            lab[2] *= lo;
            Self::from_oklab(lab)
        }
    }

    #[test]
    fn builtin_themes_count() {
        assert_eq!(builtin_themes().len(), 14);
    }

    #[test]
    fn find_theme_exact() {
        let t = find_theme("military").unwrap();
        assert_eq!(t.name, "military");
    }

    #[test]
    fn find_all_themes_by_name() {
        let names = [
            "military", "green", "amber", "alert", "cyber", "arctic", "cobalt", "void", "toxic",
            "infrared", "rose", "sepia", "walnut", "white",
        ];
        for name in names {
            assert!(find_theme(name).is_some(), "Theme '{name}' not found");
        }
    }

    #[test]
    fn find_theme_case_insensitive() {
        assert_eq!(find_theme("AMBER").unwrap().name, "amber");
        assert_eq!(find_theme("Cyber").unwrap().name, "cyber");
        assert_eq!(find_theme("WHITE").unwrap().name, "white");
    }

    #[test]
    fn find_theme_unknown() {
        assert!(find_theme("nonexistent").is_none());
    }

    #[test]
    fn all_colors_normalized() {
        for theme in builtin_themes() {
            assert!(
                (0.0..=1.0).contains(&theme.color.r)
                    && (0.0..=1.0).contains(&theme.color.g)
                    && (0.0..=1.0).contains(&theme.color.b),
                "Theme '{}' has out-of-range color",
                theme.name
            );
        }
    }

    #[test]
    fn all_wavelengths_valid() {
        for theme in builtin_themes() {
            let (lo, hi) = theme.wavelength_range;
            assert!(
                lo < hi,
                "Theme '{}' has invalid wavelength range",
                theme.name
            );
            assert!(
                lo >= 380 && hi <= 700,
                "Theme '{}' wavelength outside visible spectrum",
                theme.name
            );
        }
    }

    #[test]
    fn all_names_unique() {
        let mut names: Vec<&str> = builtin_themes().iter().map(|t| t.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), builtin_themes().len());
    }

    #[test]
    fn saturation_desaturate() {
        let green = Color::new(0.0, 1.0, 0.0);
        let muted = green.with_saturation(0.0);
        let gray = LUMA_G; // pure green luminance = LUMA_G
        assert!((muted.r - gray).abs() < 0.001);
        assert!((muted.g - gray).abs() < 0.001);
        assert!((muted.b - gray).abs() < 0.001);
    }

    #[test]
    fn saturation_unchanged() {
        let color = Color::new(0.5, 0.3, 0.8);
        let same = color.with_saturation(1.0);
        assert!((same.r - color.r).abs() < 0.001);
        assert!((same.g - color.g).abs() < 0.001);
        assert!((same.b - color.b).abs() < 0.001);
    }

    #[test]
    fn saturation_boost() {
        let color = Color::new(0.2, 0.8, 0.4);
        let vivid = color.with_saturation(1.5);
        assert!(vivid.r < color.r);
        assert!(vivid.g > color.g);
    }

    #[test]
    fn saturation_clamps() {
        let color = Color::new(0.0, 1.0, 0.0);
        let boosted = color.with_saturation(2.0);
        assert!(boosted.r >= 0.0 && boosted.r <= 1.0);
        assert!(boosted.g >= 0.0 && boosted.g <= 1.0);
        assert!(boosted.b >= 0.0 && boosted.b <= 1.0);
    }

    fn assert_color_near(a: &Color, b: &Color, tolerance: f32, msg: &str) {
        assert!(
            (a.r - b.r).abs() < tolerance
                && (a.g - b.g).abs() < tolerance
                && (a.b - b.b).abs() < tolerance,
            "{msg}: expected ({:.3}, {:.3}, {:.3}), got ({:.3}, {:.3}, {:.3})",
            b.r,
            b.g,
            b.b,
            a.r,
            a.g,
            a.b
        );
    }

    // === Inversion algorithm contract tests ===
    // These test PROPERTIES that any correct inversion must satisfy.
    // To swap algorithms, change only `invert_lightness()` above.

    fn invert(c: &Color) -> Color {
        c.invert_lightness()
    }

    #[test]
    fn invert_black_becomes_white() {
        let inv = invert(&Color::new(0.0, 0.0, 0.0));
        assert_color_near(&inv, &Color::new(1.0, 1.0, 1.0), 0.02, "black→white");
    }

    #[test]
    fn invert_white_becomes_black() {
        let inv = invert(&Color::new(1.0, 1.0, 1.0));
        assert_color_near(&inv, &Color::new(0.0, 0.0, 0.0), 0.02, "white→black");
    }

    #[test]
    fn invert_red_stays_reddish() {
        let inv = invert(&Color::new(1.0, 0.0, 0.0));
        assert!(inv.r > inv.g, "inverted red: r > g");
        assert!(inv.r > inv.b, "inverted red: r > b");
    }

    #[test]
    fn invert_yellow_stays_yellowish() {
        let inv = invert(&Color::new(1.0, 1.0, 0.0));
        assert!(
            inv.r > inv.b,
            "inverted yellow: r ({:.3}) > b ({:.3})",
            inv.r,
            inv.b
        );
        assert!(
            inv.g > inv.b,
            "inverted yellow: g ({:.3}) > b ({:.3})",
            inv.g,
            inv.b
        );
    }

    #[test]
    fn invert_blue_stays_bluish() {
        let inv = invert(&Color::new(0.0, 0.0, 1.0));
        assert!(inv.b > inv.r, "inverted blue: b > r");
        assert!(inv.b > inv.g, "inverted blue: b > g");
    }

    #[test]
    fn invert_cyan_stays_cyanish() {
        let inv = invert(&Color::new(0.0, 1.0, 1.0));
        assert!(inv.g > inv.r, "inverted cyan: g > r");
        assert!(inv.b > inv.r, "inverted cyan: b > r");
    }

    #[test]
    fn invert_preserves_gray() {
        let inv = invert(&Color::new(0.5, 0.5, 0.5));
        assert!((inv.r - inv.g).abs() < 0.02, "gray invert: r≈g");
        assert!((inv.g - inv.b).abs() < 0.02, "gray invert: g≈b");
    }

    #[test]
    fn invert_output_in_range() {
        let colors = [
            Color::new(1.0, 0.0, 0.0),
            Color::new(0.0, 1.0, 0.0),
            Color::new(0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 0.0),
            Color::new(0.5, 0.3, 0.8),
            Color::new(0.0, 0.0, 0.0),
            Color::new(1.0, 1.0, 1.0),
        ];
        for c in &colors {
            let inv = invert(c);
            assert!((0.0..=1.0).contains(&inv.r), "r out of range: {:.3}", inv.r);
            assert!((0.0..=1.0).contains(&inv.g), "g out of range: {:.3}", inv.g);
            assert!((0.0..=1.0).contains(&inv.b), "b out of range: {:.3}", inv.b);
        }
    }

    #[test]
    fn invert_reverses_luminance_direction() {
        let dark = Color::new(0.1, 0.1, 0.1);
        let bright = Color::new(0.9, 0.9, 0.9);
        let inv_dark = invert(&dark);
        let inv_bright = invert(&bright);
        let luma = |c: &Color| c.r * LUMA_R + c.g * LUMA_G + c.b * LUMA_B;
        assert!(
            luma(&inv_dark) > luma(&inv_bright),
            "dark should become brighter than bright"
        );
    }

    #[test]
    fn invert_double_preserves_lightness() {
        let colors = [
            ("red", Color::new(1.0, 0.0, 0.0)),
            ("green", Color::new(0.0, 1.0, 0.0)),
            ("blue", Color::new(0.0, 0.0, 1.0)),
            ("gray", Color::new(0.5, 0.5, 0.5)),
        ];
        for (name, c) in &colors {
            let double = invert(&invert(c));
            let orig_lab = c.to_oklab();
            let double_lab = double.to_oklab();
            assert!(
                (orig_lab[0] - double_lab[0]).abs() < 0.05,
                "{name}: lightness should round-trip, L={:.3} vs {:.3}",
                orig_lab[0],
                double_lab[0]
            );
        }
    }

    #[test]
    fn invert_double_gray_is_identity() {
        let gray = Color::new(0.5, 0.5, 0.5);
        let double = invert(&invert(&gray));
        assert_color_near(&double, &gray, 0.02, "gray double invert");
    }

    // === OKLAB conversion tests (implementation-specific) ===

    #[test]
    fn oklab_roundtrip_colors() {
        let colors = [
            ("black", Color::new(0.0, 0.0, 0.0)),
            ("white", Color::new(1.0, 1.0, 1.0)),
            ("red", Color::new(1.0, 0.0, 0.0)),
            ("green", Color::new(0.0, 1.0, 0.0)),
            ("blue", Color::new(0.0, 0.0, 1.0)),
            ("yellow", Color::new(1.0, 1.0, 0.0)),
            ("cyan", Color::new(0.0, 1.0, 1.0)),
            ("midgray", Color::new(0.5, 0.5, 0.5)),
        ];
        for (name, c) in &colors {
            let rt = Color::from_oklab(c.to_oklab());
            assert_color_near(&rt, c, 0.01, &format!("{name} roundtrip"));
        }
    }

    #[test]
    fn color_equality() {
        assert_eq!(Color::new(1.0, 0.0, 0.0), Color::new(1.0, 0.0, 0.0));
        assert_ne!(Color::new(1.0, 0.0, 0.0), Color::new(0.0, 1.0, 0.0));
    }
}
