//! The token specification — what `tangible` consumes.
//!
//! A [`Spec`] is a tree of data describing every design token: color palettes, semantic aliases,
//! typography, space, borders, shadows, glows, gradients, transitions, z-indices, opacities,
//! overlays, and the ink (contrast) configuration. It can be deserialized from JSON, TOML, YAML
//! — anywhere `serde` reaches — or constructed programmatically.
//!
//! # Example
//!
//! ```no_run
//! use tangible::Spec;
//! let json = std::fs::read_to_string("tokens.json")?;
//! let spec: Spec = serde_json::from_str(&json)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::color::{ColorDef, ColorSpace, InterpolationMode};

/// Complete token specification.
///
/// Each field corresponds to a section of generated CSS. See module docs for the
/// overall data model.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Spec {
    /// Naming and ordering scales (shades, sizes, elevations).
    pub scales: Scales,
    /// Named color palettes. Keys become CSS variable prefixes (`--colors-<name>-<shade>`).
    pub colors: BTreeMap<String, ColorDef>,
    /// Semantic aliases that map a role name (e.g. `bg`, `surface`) to a palette/shade or raw value.
    pub semantic: BTreeMap<String, SemanticValue>,
    /// Typography tokens: families, sizes, weights, line-heights, letter-spacings.
    pub fonts: Fonts,
    /// Space scale values, indexed by `scales.sizes`.
    pub space: Vec<String>,
    /// Dimension scale values, indexed by `scales.sizes`.
    pub dimensions: Vec<String>,
    /// Border radius (named) and widths (positional).
    pub borders: Borders,
    /// Box-shadow palettes and elevation profiles.
    pub shadows: Shadows,
    /// Text-shadow elevations, keyed by name (typically `low` / `medium` / `high`).
    #[serde(rename = "textShadows")]
    pub text_shadows: BTreeMap<String, TextShadowElevation>,
    /// Ink (contrast) configuration — the light/dark text colors used for automatic contrast.
    pub ink: InkConfig,
    /// Per-palette glow definitions.
    pub glows: BTreeMap<String, Glow>,
    /// CSS gradients composed from palette-anchored stops.
    pub gradients: Vec<Gradient>,
    /// Named transition strings (e.g. `default`, `slow`).
    pub transitions: BTreeMap<String, String>,
    /// Named z-index values (numbers or strings).
    pub z: BTreeMap<String, serde_json::Value>,
    /// Named opacity values (0.0–1.0).
    pub opacity: BTreeMap<String, f64>,
    /// Overlay color tokens (e.g. modal scrim colors).
    #[serde(default)]
    pub overlay: BTreeMap<String, String>,
}

/// A semantic color value — either a reference to a palette shade or a raw CSS value.
///
/// The JSON shape is either `["palette", 500]` (palette ref) or a plain string (raw value).
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum SemanticValue {
    /// Reference to a palette shade: `[palette_name, shade]`.
    PaletteRef(String, u32),
    /// Raw CSS value (for overrides like AAA-adjusted muted text).
    Raw(String),
}

/// Naming and ordering scales used across the spec.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Scales {
    /// Numeric shade labels for palettes (e.g. `[100, 200, …, 900]`).
    pub shades: Vec<u32>,
    /// Size names used to index `space`, `dimensions`, and `fonts.sizes` (e.g. `["sm", "md", "lg"]`).
    pub sizes: Vec<String>,
    /// Elevation names used to index glow radii (e.g. `["low", "medium", "high"]`).
    pub elevations: Vec<String>,
}

/// Typography tokens.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Fonts {
    /// Font families, keyed by role (e.g. `sans`, `mono`).
    pub families: BTreeMap<String, Vec<String>>,
    /// Font sizes, indexed by `scales.sizes`.
    pub sizes: Vec<String>,
    /// Font weights (numeric, e.g. `[400, 700]`).
    pub weights: Vec<u32>,
    /// Line heights, indexed by `scales.sizes`.
    #[serde(rename = "lineHeights")]
    pub line_heights: Vec<f64>,
    /// Letter spacings, keyed by name (e.g. `tight`, `wide`).
    #[serde(rename = "letterSpacing")]
    pub letter_spacing: BTreeMap<String, String>,
}

/// Border tokens.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Borders {
    /// Border radius values, keyed by name (e.g. `sm`, `md`, `lg`).
    pub radius: BTreeMap<String, String>,
    /// Border widths in ascending order (positional — names are derived).
    pub widths: Vec<String>,
}

/// Box-shadow tokens.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Shadows {
    /// Shadow colors (HSL components as a string), keyed by palette name.
    pub colors: BTreeMap<String, String>,
    /// Elevation profiles, keyed by name.
    pub elevations: BTreeMap<String, Elevation>,
}

/// A single shadow elevation profile.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Elevation {
    /// Number of layered shadows (informational; offsets array is authoritative).
    pub layers: u32,
    /// `(x, y)` offsets for each layer.
    pub offsets: Vec<(f64, f64)>,
    /// Layer opacity.
    pub opacity: f64,
}

/// Per-palette glow definition.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Glow {
    /// Color template, with `{a}` as the alpha placeholder (e.g. `hsla(170, 100%, 40%, {a})`).
    pub color: String,
    /// Blur radii, indexed by `scales.elevations`.
    pub radii: Vec<u32>,
}

/// Text-shadow elevation profile.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TextShadowElevation {
    /// Blur radii per layer.
    pub blur: Vec<u32>,
    /// Layer opacity.
    pub opacity: f64,
}

/// Ink (contrast) configuration.
///
/// Both fields are `(palette_name, shade)` references into [`Spec::colors`] and [`Scales::shades`].
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InkConfig {
    /// The light-ink color (typically near-white).
    pub light: (String, u32),
    /// The dark-ink color (typically near-black).
    pub dark: (String, u32),
}

/// A CSS gradient defined from palette-anchored stops.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Gradient {
    /// Gradient name (becomes part of `--gradient-<name>`).
    pub name: String,
    /// CSS gradient type (e.g. `linear`, `radial`, `conic`).
    #[serde(rename = "type")]
    pub gradient_type: String,
    /// Gradient angle in degrees.
    pub angle: u32,
    /// Stops as `(palette_name, shade)` pairs.
    pub stops: Vec<(String, u32)>,
    /// Optional blend color space. If set, the gradient is pre-resolved to hex stops; otherwise
    /// the gradient references CSS variables directly.
    #[serde(default)]
    pub blend: Option<ColorSpace>,
    /// Optional interpolation mode, used only when `blend` is set.
    #[serde(default)]
    pub mode: Option<InterpolationMode>,
    /// Number of evenly-spaced samples to emit when `blend` is set.
    #[serde(default = "default_gradient_samples")]
    pub samples: usize,
}

fn default_gradient_samples() -> usize {
    7
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_json() -> &'static str {
        r#"{
          "scales": { "shades": [100], "sizes": ["sm"], "elevations": ["low"] },
          "colors": {},
          "semantic": {},
          "fonts": {
            "families": {},
            "sizes": ["1rem"],
            "weights": [400],
            "lineHeights": [1.5],
            "letterSpacing": {}
          },
          "space": ["1rem"],
          "dimensions": ["1rem"],
          "borders": { "radius": {}, "widths": [] },
          "shadows": { "colors": {}, "elevations": {} },
          "textShadows": {},
          "ink": { "light": ["x", 0], "dark": ["x", 0] },
          "glows": {},
          "gradients": [],
          "transitions": {},
          "z": {},
          "opacity": {}
        }"#
    }

    #[test]
    fn spec_deserializes_with_minimal_fields() {
        let spec: Spec = serde_json::from_str(minimal_json()).unwrap();
        assert_eq!(spec.scales.shades, vec![100]);
        assert!(spec.overlay.is_empty());
    }

    #[test]
    fn spec_overlay_defaults_to_empty() {
        let spec: Spec = serde_json::from_str(minimal_json()).unwrap();
        assert!(spec.overlay.is_empty());
    }

    #[test]
    fn spec_roundtrips_through_serde() {
        let spec: Spec = serde_json::from_str(minimal_json()).unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        let again: Spec = serde_json::from_str(&json).unwrap();
        assert_eq!(again.scales.shades, vec![100]);
    }

    #[test]
    fn semantic_value_palette_ref_deserializes() {
        let v: SemanticValue = serde_json::from_str(r#"["primary", 500]"#).unwrap();
        match v {
            SemanticValue::PaletteRef(name, shade) => {
                assert_eq!(name, "primary");
                assert_eq!(shade, 500);
            }
            SemanticValue::Raw(_) => panic!("expected PaletteRef"),
        }
    }

    #[test]
    fn semantic_value_raw_deserializes() {
        let v: SemanticValue = serde_json::from_str(r##""#abcdef""##).unwrap();
        match v {
            SemanticValue::Raw(s) => assert_eq!(s, "#abcdef"),
            SemanticValue::PaletteRef(_, _) => panic!("expected Raw"),
        }
    }
}
