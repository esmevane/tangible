//! Build a minimal token spec entirely in Rust (no JSON) and render it to CSS.
//!
//! Run with `cargo run --example programmatic`.

#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::error::Error;

use tangible::color::{ColorDef, ColorSpace, GradientDef, GradientStop, InterpolationMode};
use tangible::spec::{
    Borders, Elevation, Fonts, Glow, Gradient, InkConfig, Scales, SemanticValue, Shadows,
    TextShadowElevation,
};
use tangible::{Renderer, Spec};

fn main() -> Result<(), Box<dyn Error>> {
    let mut colors = BTreeMap::new();
    colors.insert(
        "neutral".into(),
        ColorDef::Explicit(vec!["#f5f5f5".into(), "#888888".into(), "#1a1a1a".into()]),
    );
    colors.insert(
        "primary".into(),
        ColorDef::Gradient(GradientDef {
            stops: vec![
                GradientStop::Color("#b3f0e6".into()),
                GradientStop::Color("#00c9a7".into()),
                GradientStop::Color("#061e1c".into()),
            ],
            mode: InterpolationMode::Linear,
            blend: ColorSpace::Oklab,
        }),
    );

    let mut semantic = BTreeMap::new();
    semantic.insert(
        "bg".into(),
        SemanticValue::PaletteRef("neutral".into(), 900),
    );
    semantic.insert(
        "text".into(),
        SemanticValue::PaletteRef("neutral".into(), 100),
    );
    semantic.insert(
        "interactive".into(),
        SemanticValue::PaletteRef("primary".into(), 500),
    );

    let mut shadow_elevations = BTreeMap::new();
    shadow_elevations.insert(
        "low".into(),
        Elevation {
            layers: 1,
            offsets: vec![(0.0, 1.0)],
            opacity: 0.1,
        },
    );

    let mut shadow_colors = BTreeMap::new();
    shadow_colors.insert("neutral".into(), "220 10% 10%".into());

    let mut text_shadows = BTreeMap::new();
    text_shadows.insert(
        "medium".into(),
        TextShadowElevation {
            blur: vec![2, 4],
            opacity: 0.4,
        },
    );

    let mut glows = BTreeMap::new();
    glows.insert(
        "primary".into(),
        Glow {
            color: "hsla(170, 100%, 40%, {a})".into(),
            radii: vec![4, 8, 16],
        },
    );

    let mut transitions = BTreeMap::new();
    transitions.insert("default".into(), "150ms ease".into());

    let mut z = BTreeMap::new();
    z.insert("base".into(), serde_json::json!(0));

    let mut opacity = BTreeMap::new();
    opacity.insert("disabled".into(), 0.4);

    let mut letter_spacing = BTreeMap::new();
    letter_spacing.insert("tight".into(), "-0.01em".into());

    let mut families = BTreeMap::new();
    families.insert("sans".into(), vec!["Inter".into(), "sans-serif".into()]);

    let mut radius = BTreeMap::new();
    radius.insert("sm".into(), "4px".into());
    radius.insert("md".into(), "8px".into());
    radius.insert("lg".into(), "16px".into());

    let spec = Spec {
        scales: Scales {
            shades: vec![100, 500, 900],
            sizes: vec!["sm".into(), "md".into(), "lg".into()],
            elevations: vec!["low".into(), "medium".into(), "high".into()],
        },
        colors,
        semantic,
        fonts: Fonts {
            families,
            sizes: vec!["0.75rem".into(), "1rem".into(), "1.25rem".into()],
            weights: vec![400, 700],
            line_heights: vec![1.4, 1.5, 1.6],
            letter_spacing,
        },
        space: vec!["0.5rem".into(), "1rem".into(), "2rem".into()],
        dimensions: vec!["16rem".into(), "32rem".into(), "64rem".into()],
        borders: Borders {
            radius,
            widths: vec!["1px".into(), "2px".into(), "4px".into()],
        },
        shadows: Shadows {
            colors: shadow_colors,
            elevations: shadow_elevations,
        },
        text_shadows,
        ink: InkConfig {
            light: ("neutral".into(), 100),
            dark: ("neutral".into(), 900),
        },
        glows,
        gradients: vec![Gradient {
            name: "sweep".into(),
            gradient_type: "linear".into(),
            angle: 135,
            stops: vec![("primary".into(), 100), ("primary".into(), 900)],
            blend: Some(ColorSpace::Oklab),
            mode: Some(InterpolationMode::Linear),
            samples: 5,
        }],
        transitions,
        z,
        opacity,
        overlay: BTreeMap::new(),
    };

    let manifest = Renderer::new().render(&spec)?;
    print!("{manifest}");
    Ok(())
}
