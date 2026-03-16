# Focus
[![CI](https://github.com/i-am-logger/focus/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/i-am-logger/focus/actions/workflows/ci.yml)
[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Hyprland](https://img.shields.io/badge/Hyprland-58E1FF?logo=hyprland&logoColor=white)](https://hyprland.org/)

> Monochromatic screen overlay for Hyprland.

Focus overlays your entire screen with shades of a single color palette, creating an immersive monochromatic experience. Choose a theme and your desktop transforms — military green night-vision, amber warmth, cyber cyan, and more.

Built as a Rust CLI that generates GLSL shaders and applies them via Hyprland's native `decoration:screen_shader`. Works on all content including fullscreen apps. Launch it from your keybinds.

| | | | |
|---|---|---|---|
| ![military](docs/military.png) | ![amber](docs/amber.png) | ![cyber](docs/cyber.png) | ![white](docs/white.jpeg) |
| **military** | **amber** | **cyber** | **white** |

## Themes

| Theme | Color | Wavelength | Vibe |
|-------|-------|------------|------|
| **military** | Green | 530–560 nm | Night-vision, tactical |
| **green** | Green | 520–530 nm | Classic P1 green CRT |
| **amber** | Amber | 598–608 nm | Classic P3 amber CRT |
| **alert** | Red | 620–680 nm | Warning lights, emergency |
| **cyber** | Cyan | 485–500 nm | Neon, futuristic |
| **arctic** | Blue | 460–480 nm | Cold, ice |
| **cobalt** | Deep blue | 450–470 nm | Industrial |
| **void** | Purple | 400–430 nm | Deep, cosmic |
| **toxic** | Yellow-green | 550–570 nm | Radioactive |
| **infrared** | Magenta | 620–700 nm | Thermal camera |
| **rose** | Pink | 600–650 nm | Soft, lo-fi |
| **sepia** | Brown | 580–620 nm | Old photograph |
| **walnut** | Brown | 580–610 nm | Dark stained wood |
| **white** | White | 380–700 nm | Classic P4 white CRT |

## Usage

```bash
# Apply a theme
focus --theme military

# Apply with reduced intensity (50% blend)
focus --theme cyber --opacity 0.5

# Dim the output
focus --theme amber --brightness 0.5

# Boost color vividness
focus --theme sepia --saturation 1.5

# Combine adjustments
focus --theme void --opacity 0.8 --brightness 1.2 --saturation 1.3

# List available themes
focus --list

# Turn off
focus --off
```

### Hyprland Keybinds

```ini
bind = $mainMod, F, exec, focus --theme military
bind = $mainMod SHIFT, F, exec, focus --off
```

## How It Works

Focus generates a GLSL fragment shader that converts each pixel to luminance (using Rec. 709 coefficients) and maps it to the theme color. The shader is applied at the compositor level via `hyprctl keyword decoration:screen_shader`, so it works on everything — including fullscreen apps and video.

| Flag | Range | Effect |
|------|-------|--------|
| `--opacity` | 0.0–1.0 | Blend with original colors (1.0 = full monochrome) |
| `--brightness` | 0.1–2.0 | Dim or boost the tint output |
| `--saturation` | 0.0–2.0 | Mute (toward gray) or vivify the theme color |

## Requirements

- Hyprland
- Rust Edition 2024

## Development

```bash
# Enter devenv shell
direnv allow

# Build
dev-build

# Run
dev-run

# Test
dev-test
```

## License

Creative Commons Attribution-NonCommercial-ShareAlike (CC BY-NC-SA) 4.0 International

See [LICENSE](LICENSE) for details.
