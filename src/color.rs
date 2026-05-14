//! Color definitions, gradients, and sampling.
//!
//! A palette in `tangible` is either an explicit list of hex strings or a [`GradientDef`] that
//! describes a color ramp to be sampled at runtime. Either form is captured by [`ColorDef`], and
//! both resolve to the same shape — a `Vec<String>` of hex colors — through [`resolve_colors`].
//!
//! Gradients can be interpolated in one of three modes ([`InterpolationMode`]) and blended in one
//! of four color spaces ([`ColorSpace`]). Stops may be bare colors or pinned to a position
//! ([`GradientStop`], [`Position`]).

use colorgrad::{
    BasisGradient, BlendMode, CatmullRomGradient, Gradient as GradientSampler, GradientBuilder,
    LinearGradient,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::error::Error;

/// A palette definition — either an explicit list of hex colors or an interpolated gradient.
///
/// `ColorDef` deserializes from either a JSON array (`["#ff0000", "#00ff00"]`) or a gradient
/// object (`{ "stops": [...], "mode": "linear", "blend": "oklab" }`).
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ColorDef {
    /// An explicit list of hex colors, one per shade.
    Explicit(Vec<String>),
    /// A gradient sampled at the spec's shade count.
    Gradient(GradientDef),
}

/// A percentage position in a gradient, in the closed range `0..=100`.
#[derive(Clone, Debug, PartialEq)]
pub struct Position(pub f32);

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Strip trailing zeros: 50.00 → "50%", 33.50 → "33.5%". The cast is bounded by
        // `FromStr` to 0..=100, so it cannot truncate or underflow.
        if (self.0.fract() - 0.0).abs() < f32::EPSILON {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let whole = self.0 as u32;
            write!(f, "{whole}%")
        } else {
            write!(f, "{}%", self.0)
        }
    }
}

impl FromStr for Position {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let Some(num) = trimmed.strip_suffix('%') else {
            return Err(Error::InvalidPosition(
                s.to_string(),
                "must end with '%'".into(),
            ));
        };
        let val: f32 = num.trim().parse().map_err(|e: std::num::ParseFloatError| {
            Error::InvalidPosition(s.to_string(), e.to_string())
        })?;
        if !(0.0..=100.0).contains(&val) {
            return Err(Error::InvalidPosition(
                s.to_string(),
                format!("out of range 0–100: {val}"),
            ));
        }
        Ok(Position(val))
    }
}

/// A single stop in a gradient: either a bare color or a color pinned to a position.
#[derive(Clone, Debug, PartialEq)]
pub enum GradientStop {
    /// A color with no explicit position (distributed evenly along the gradient).
    Color(String),
    /// A color pinned to a specific position.
    Positioned(String, Position),
}

impl GradientStop {
    /// Return the stop's color string, regardless of variant.
    #[must_use]
    pub fn color(&self) -> &str {
        match self {
            GradientStop::Color(c) | GradientStop::Positioned(c, _) => c,
        }
    }

    /// Return the stop's pinned position, if any.
    #[must_use]
    pub fn position(&self) -> Option<&Position> {
        match self {
            GradientStop::Color(_) => None,
            GradientStop::Positioned(_, p) => Some(p),
        }
    }
}

impl fmt::Display for GradientStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GradientStop::Color(c) => write!(f, "{c}"),
            GradientStop::Positioned(c, p) => write!(f, "{c} {p}"),
        }
    }
}

impl FromStr for GradientStop {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidGradientStop(
                s.to_string(),
                "empty gradient stop".into(),
            ));
        }
        // Look for a trailing percentage and split color from position on the preceding space.
        if let Some(pct_idx) = trimmed.rfind('%') {
            let before_pct = &trimmed[..=pct_idx];
            if let Some(space_idx) = before_pct.rfind(' ') {
                let color = trimmed[..space_idx].trim();
                let pos_str = trimmed[space_idx..].trim();
                if !color.is_empty() {
                    let pos = pos_str.parse::<Position>()?;
                    return Ok(GradientStop::Positioned(color.to_string(), pos));
                }
            }
        }
        Ok(GradientStop::Color(trimmed.to_string()))
    }
}

impl Serialize for GradientStop {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GradientStop {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A gradient definition — a sequence of stops with optional interpolation and blend overrides.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GradientDef {
    /// The stops anchoring the gradient.
    pub stops: Vec<GradientStop>,
    /// Interpolation curve. Defaults to [`InterpolationMode::Linear`].
    #[serde(default)]
    pub mode: InterpolationMode,
    /// Color space the interpolation runs in. Defaults to [`ColorSpace::Rgb`].
    #[serde(default)]
    pub blend: ColorSpace,
}

/// Interpolation curves for gradient sampling.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum InterpolationMode {
    /// Straight-line interpolation between adjacent stops.
    #[default]
    Linear,
    /// Catmull–Rom spline — smoothly curved through the stops.
    CatmullRom,
    /// Basis spline — smoothly curved but does not pass through interior stops.
    Basis,
}

/// Color space in which a gradient is interpolated.
///
/// Choice of color space affects the *midpoints* of a gradient — endpoints are always identical.
/// Perceptually uniform spaces ([`Oklab`](Self::Oklab), [`Lab`](Self::Lab)) typically produce
/// more natural-looking ramps than naïve RGB.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSpace {
    /// Standard sRGB.
    #[default]
    Rgb,
    /// Linearized sRGB.
    LinearRgb,
    /// Perceptually uniform Oklab.
    Oklab,
    /// CIE Lab.
    Lab,
}

impl ColorSpace {
    fn to_blend_mode(&self) -> BlendMode {
        match self {
            ColorSpace::Rgb => BlendMode::Rgb,
            ColorSpace::LinearRgb => BlendMode::LinearRgb,
            ColorSpace::Oklab => BlendMode::Oklab,
            ColorSpace::Lab => BlendMode::Lab,
        }
    }
}

/// Resolve a [`ColorDef`] into concrete hex values by sampling the gradient at `count`
/// evenly-spaced positions.
///
/// For an [`ColorDef::Explicit`] palette, the inner list is returned unchanged. For a
/// [`ColorDef::Gradient`], the gradient is built (honoring `mode`, `blend`, and any positioned
/// stops) and sampled at `count` points.
///
/// # Errors
///
/// Returns [`Error::InvalidGradient`] if the gradient could not be built from its stops.
///
/// [`Error::InvalidGradient`]: crate::Error::InvalidGradient
pub(crate) fn resolve_colors(def: &ColorDef, count: usize) -> Result<Vec<String>, Error> {
    match def {
        ColorDef::Explicit(hexes) => Ok(hexes.clone()),
        ColorDef::Gradient(gdef) => sample_gradient_def(gdef, count),
    }
}

fn sample_gradient_def(gdef: &GradientDef, count: usize) -> Result<Vec<String>, Error> {
    let colors: Vec<&str> = gdef.stops.iter().map(GradientStop::color).collect();
    let mut builder = GradientBuilder::new();
    builder.html_colors(&colors);
    builder.mode(gdef.blend.to_blend_mode());

    // If any stops have explicit positions, set the domain so the sampler honors them.
    let has_positions = gdef.stops.iter().any(|s| s.position().is_some());
    if has_positions {
        let domain: Vec<f32> = gdef
            .stops
            .iter()
            .enumerate()
            .map(|(i, stop)| {
                stop.position().map_or_else(
                    || {
                        if gdef.stops.len() <= 1 {
                            0.0
                        } else {
                            #[allow(clippy::cast_precision_loss)]
                            let value = i as f32 / (gdef.stops.len() - 1) as f32;
                            value
                        }
                    },
                    |p| p.0 / 100.0,
                )
            })
            .collect();
        builder.domain(&domain);
    }

    build_and_sample(&mut builder, &gdef.mode, count, "")
}

fn build_and_sample(
    builder: &mut GradientBuilder,
    mode: &InterpolationMode,
    count: usize,
    name: &str,
) -> Result<Vec<String>, Error> {
    match mode {
        InterpolationMode::Linear => {
            let grad = builder
                .build::<LinearGradient>()
                .map_err(|e| Error::InvalidGradient {
                    name: name.to_string(),
                    message: e.to_string(),
                })?;
            Ok(sample(&grad, count))
        }
        InterpolationMode::CatmullRom => {
            let grad =
                builder
                    .build::<CatmullRomGradient>()
                    .map_err(|e| Error::InvalidGradient {
                        name: name.to_string(),
                        message: e.to_string(),
                    })?;
            Ok(sample(&grad, count))
        }
        InterpolationMode::Basis => {
            let grad = builder
                .build::<BasisGradient>()
                .map_err(|e| Error::InvalidGradient {
                    name: name.to_string(),
                    message: e.to_string(),
                })?;
            Ok(sample(&grad, count))
        }
    }
}

fn sample(grad: &impl GradientSampler, count: usize) -> Vec<String> {
    if count <= 1 {
        return vec![grad.at(0.0).to_css_hex()];
    }
    (0..count)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / (count - 1) as f32;
            grad.at(t).to_css_hex()
        })
        .collect()
}

/// Resolve every color definition in a palette map, returning palette name → hex values.
///
/// # Errors
///
/// Returns [`Error::InvalidGradient`] if any gradient palette could not be sampled.
///
/// [`Error::InvalidGradient`]: crate::Error::InvalidGradient
pub(crate) fn resolve_all_colors(
    colors: &BTreeMap<String, ColorDef>,
    count: usize,
) -> Result<BTreeMap<String, Vec<String>>, Error> {
    colors
        .iter()
        .map(|(name, def)| resolve_colors(def, count).map(|hexes| (name.clone(), hexes)))
        .collect()
}

/// A CSS-gradient definition that references palette/shade pairs as its anchor points.
///
/// `Gradient` lives in [`crate::spec`] structurally but is referenced from here for the
/// resolution helper [`resolve_gradient_stops`].
pub(crate) use crate::spec::Gradient;

pub(crate) fn resolve_gradient_stops(
    grad: &Gradient,
    resolved: &BTreeMap<String, Vec<String>>,
    shades: &[u32],
) -> Result<Vec<String>, Error> {
    let mut anchor_hexes: Vec<&str> = Vec::with_capacity(grad.stops.len());
    for (palette, shade) in &grad.stops {
        let palette_colors = resolved.get(palette).ok_or_else(|| Error::UnknownPalette {
            palette: palette.clone(),
            context: format!(" in gradient {:?}", grad.name),
        })?;
        let shade_idx =
            shades
                .iter()
                .position(|s| s == shade)
                .ok_or_else(|| Error::UnknownShade {
                    shade: *shade,
                    context: format!(" in gradient {:?}", grad.name),
                })?;
        anchor_hexes.push(palette_colors[shade_idx].as_str());
    }

    let blend = grad.blend.clone().unwrap_or_default();
    let mode = grad.mode.clone().unwrap_or_default();

    let mut builder = GradientBuilder::new();
    builder.html_colors(&anchor_hexes);
    builder.mode(blend.to_blend_mode());

    build_and_sample(&mut builder, &mode, grad.samples, &grad.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(color: &str) -> GradientStop {
        GradientStop::Color(color.into())
    }

    fn stop_at(color: &str, pct: f32) -> GradientStop {
        GradientStop::Positioned(color.into(), Position(pct))
    }

    #[test]
    fn explicit_colors_pass_through() {
        let def = ColorDef::Explicit(vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()]);
        let resolved = resolve_colors(&def, 3).unwrap();
        assert_eq!(resolved, vec!["#ff0000", "#00ff00", "#0000ff"]);
    }

    #[test]
    fn gradient_produces_correct_count() {
        let def = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#000000"), stop("#ffffff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::default(),
        });
        let resolved = resolve_colors(&def, 12).unwrap();
        assert_eq!(resolved.len(), 12);
    }

    #[test]
    fn gradient_endpoints_match_stops() {
        let def = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#000000"), stop("#ffffff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::default(),
        });
        let resolved = resolve_colors(&def, 5).unwrap();
        assert_eq!(resolved.first().unwrap(), "#000000");
        assert_eq!(resolved.last().unwrap(), "#ffffff");
    }

    #[test]
    fn gradient_supports_css_hex_formats() {
        let def = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#000"), stop("#fff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::default(),
        });
        let resolved = resolve_colors(&def, 3).unwrap();
        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved.first().unwrap(), "#000000");
    }

    #[test]
    fn gradient_def_deserializes_from_json() {
        let json = r##"{ "stops": ["#000", "#fff"], "mode": "catmull-rom" }"##;
        let def: ColorDef = serde_json::from_str(json).unwrap();
        match &def {
            ColorDef::Gradient(g) => {
                assert_eq!(g.stops.len(), 2);
                assert_eq!(g.mode, InterpolationMode::CatmullRom);
            }
            ColorDef::Explicit(_) => panic!("expected Gradient variant"),
        }
    }

    #[test]
    fn explicit_array_deserializes_from_json() {
        let json = r##"["#ff0000", "#00ff00"]"##;
        let def: ColorDef = serde_json::from_str(json).unwrap();
        match &def {
            ColorDef::Explicit(v) => assert_eq!(v.len(), 2),
            ColorDef::Gradient(_) => panic!("expected Explicit variant"),
        }
    }

    #[test]
    fn gradient_mode_defaults_to_linear() {
        let json = r##"{ "stops": ["#000", "#fff"] }"##;
        let def: ColorDef = serde_json::from_str(json).unwrap();
        match &def {
            ColorDef::Gradient(g) => assert_eq!(g.mode, InterpolationMode::Linear),
            ColorDef::Explicit(_) => panic!("expected Gradient variant"),
        }
    }

    #[test]
    fn all_interpolation_modes_produce_output() {
        for mode in [
            InterpolationMode::Linear,
            InterpolationMode::CatmullRom,
            InterpolationMode::Basis,
        ] {
            let def = ColorDef::Gradient(GradientDef {
                stops: vec![stop("#b3f0e6"), stop("#00c9a7"), stop("#061e1c")],
                mode,
                blend: ColorSpace::default(),
            });
            let resolved = resolve_colors(&def, 12).unwrap();
            assert_eq!(resolved.len(), 12);
            assert!(resolved.iter().all(|h| h.starts_with('#')));
        }
    }

    #[test]
    fn position_parse_roundtrip() {
        let p: Position = "50%".parse().unwrap();
        assert!((p.0 - 50.0).abs() < f32::EPSILON);
        assert_eq!(p.to_string(), "50%");
    }

    #[test]
    fn position_parse_fractional() {
        let p: Position = "33.5%".parse().unwrap();
        assert!((p.0 - 33.5).abs() < f32::EPSILON);
        assert_eq!(p.to_string(), "33.5%");
    }

    #[test]
    fn position_rejects_missing_percent() {
        let err = "50".parse::<Position>().unwrap_err();
        assert!(matches!(err, Error::InvalidPosition(_, _)));
    }

    #[test]
    fn position_rejects_out_of_range() {
        assert!("101%".parse::<Position>().is_err());
        assert!("-1%".parse::<Position>().is_err());
    }

    #[test]
    fn gradient_stop_parse_bare_color() {
        let stop: GradientStop = "#ff0000".parse().unwrap();
        assert_eq!(stop, GradientStop::Color("#ff0000".into()));
        assert_eq!(stop.to_string(), "#ff0000");
    }

    #[test]
    fn gradient_stop_parse_positioned() {
        let stop: GradientStop = "#ff0000 25%".parse().unwrap();
        assert_eq!(
            stop,
            GradientStop::Positioned("#ff0000".into(), Position(25.0))
        );
        assert_eq!(stop.to_string(), "#ff0000 25%");
    }

    #[test]
    fn gradient_stop_accessors() {
        let bare: GradientStop = "#aaa".parse().unwrap();
        assert_eq!(bare.color(), "#aaa");
        assert!(bare.position().is_none());

        let pinned: GradientStop = "#bbb 75%".parse().unwrap();
        assert_eq!(pinned.color(), "#bbb");
        assert!((pinned.position().unwrap().0 - 75.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gradient_stop_serde_roundtrip() {
        let stop = GradientStop::Positioned("#ff0000".into(), Position(25.0));
        let json = serde_json::to_string(&stop).unwrap();
        assert_eq!(json, r##""#ff0000 25%""##);
        let back: GradientStop = serde_json::from_str(&json).unwrap();
        assert_eq!(back, stop);
    }

    #[test]
    fn gradient_def_with_positions_deserializes() {
        let json = r##"{ "stops": ["#000 0%", "#888 30%", "#fff 100%"] }"##;
        let def: ColorDef = serde_json::from_str(json).unwrap();
        match &def {
            ColorDef::Gradient(g) => {
                assert_eq!(g.stops.len(), 3);
                assert_eq!(g.stops[1].color(), "#888");
                assert!((g.stops[1].position().unwrap().0 - 30.0).abs() < f32::EPSILON);
            }
            ColorDef::Explicit(_) => panic!("expected Gradient variant"),
        }
    }

    #[test]
    fn blend_defaults_to_rgb() {
        let json = r##"{ "stops": ["#000", "#fff"] }"##;
        let def: ColorDef = serde_json::from_str(json).unwrap();
        match &def {
            ColorDef::Gradient(g) => assert_eq!(g.blend, ColorSpace::Rgb),
            ColorDef::Explicit(_) => panic!("expected Gradient variant"),
        }
    }

    #[test]
    fn blend_deserializes_all_variants() {
        for (name, expected) in [
            ("rgb", ColorSpace::Rgb),
            ("linear-rgb", ColorSpace::LinearRgb),
            ("oklab", ColorSpace::Oklab),
            ("lab", ColorSpace::Lab),
        ] {
            let json = format!(r##"{{ "stops": ["#000", "#fff"], "blend": "{name}" }}"##);
            let def: ColorDef = serde_json::from_str(&json).unwrap();
            match &def {
                ColorDef::Gradient(g) => assert_eq!(g.blend, expected, "failed for {name}"),
                ColorDef::Explicit(_) => panic!("expected Gradient variant"),
            }
        }
    }

    #[test]
    fn all_blend_modes_produce_output() {
        for blend in [
            ColorSpace::Rgb,
            ColorSpace::LinearRgb,
            ColorSpace::Oklab,
            ColorSpace::Lab,
        ] {
            let def = ColorDef::Gradient(GradientDef {
                stops: vec![stop("#ff0000"), stop("#0000ff")],
                mode: InterpolationMode::Linear,
                blend,
            });
            let resolved = resolve_colors(&def, 5).unwrap();
            assert_eq!(resolved.len(), 5);
            assert!(resolved.iter().all(|h| h.starts_with('#')));
        }
    }

    #[test]
    fn oklab_produces_different_midpoints_than_rgb() {
        let rgb = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#ff0000"), stop("#0000ff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::Rgb,
        });
        let oklab = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#ff0000"), stop("#0000ff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::Oklab,
        });
        let rgb_colors = resolve_colors(&rgb, 5).unwrap();
        let oklab_colors = resolve_colors(&oklab, 5).unwrap();
        assert_eq!(rgb_colors[0], oklab_colors[0]);
        assert_eq!(rgb_colors[4], oklab_colors[4]);
        assert_ne!(rgb_colors[2], oklab_colors[2]);
    }

    #[test]
    fn positioned_stops_affect_sampling() {
        let biased = ColorDef::Gradient(GradientDef {
            stops: vec![
                stop_at("#000000", 0.0),
                stop_at("#808080", 10.0),
                stop_at("#ffffff", 100.0),
            ],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::default(),
        });
        let even = ColorDef::Gradient(GradientDef {
            stops: vec![stop("#000000"), stop("#808080"), stop("#ffffff")],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::default(),
        });
        let biased_colors = resolve_colors(&biased, 5).unwrap();
        let even_colors = resolve_colors(&even, 5).unwrap();
        assert_ne!(biased_colors, even_colors);
    }

    fn test_resolved_palettes() -> BTreeMap<String, Vec<String>> {
        let mut m = BTreeMap::new();
        m.insert(
            "primary".into(),
            vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()],
        );
        m.insert(
            "accent".into(),
            vec!["#ffff00".into(), "#ff00ff".into(), "#00ffff".into()],
        );
        m
    }

    fn test_shades() -> Vec<u32> {
        vec![100, 200, 300]
    }

    #[test]
    fn resolve_gradient_stops_produces_correct_count() {
        let grad = Gradient {
            name: "test".into(),
            gradient_type: "linear".into(),
            angle: 135,
            stops: vec![("primary".into(), 100), ("accent".into(), 300)],
            blend: Some(ColorSpace::Oklab),
            mode: None,
            samples: 7,
        };
        let resolved = test_resolved_palettes();
        let stops = resolve_gradient_stops(&grad, &resolved, &test_shades()).unwrap();
        assert_eq!(stops.len(), 7);
        assert!(stops.iter().all(|s| s.starts_with('#')));
    }

    #[test]
    fn resolve_gradient_stops_endpoints_match_anchors() {
        let grad = Gradient {
            name: "test".into(),
            gradient_type: "linear".into(),
            angle: 135,
            stops: vec![("primary".into(), 100), ("accent".into(), 300)],
            blend: Some(ColorSpace::Rgb),
            mode: Some(InterpolationMode::Linear),
            samples: 5,
        };
        let resolved = test_resolved_palettes();
        let stops = resolve_gradient_stops(&grad, &resolved, &test_shades()).unwrap();
        assert_eq!(stops[0], "#ff0000");
        assert_eq!(stops[4], "#00ffff");
    }

    #[test]
    fn resolve_gradient_stops_blend_affects_midpoints() {
        let make = |blend: ColorSpace| {
            let grad = Gradient {
                name: "test".into(),
                gradient_type: "linear".into(),
                angle: 135,
                stops: vec![("primary".into(), 100), ("accent".into(), 300)],
                blend: Some(blend),
                mode: Some(InterpolationMode::Linear),
                samples: 5,
            };
            resolve_gradient_stops(&grad, &test_resolved_palettes(), &test_shades()).unwrap()
        };
        let rgb = make(ColorSpace::Rgb);
        let oklab = make(ColorSpace::Oklab);
        assert_eq!(rgb[0], oklab[0]);
        assert_eq!(rgb[4], oklab[4]);
        assert_ne!(rgb[2], oklab[2]);
    }

    #[test]
    fn resolve_gradient_stops_unknown_palette_errors() {
        let grad = Gradient {
            name: "test".into(),
            gradient_type: "linear".into(),
            angle: 0,
            stops: vec![("missing".into(), 100)],
            blend: Some(ColorSpace::Rgb),
            mode: None,
            samples: 3,
        };
        let err =
            resolve_gradient_stops(&grad, &test_resolved_palettes(), &test_shades()).unwrap_err();
        assert!(matches!(err, Error::UnknownPalette { .. }));
    }
}
