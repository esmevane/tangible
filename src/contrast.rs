//! WCAG contrast computation and ink selection.
//!
//! For every semantic color, `tangible` picks the higher-contrast "ink" (text) color from a pair
//! of light/dark candidates. Selection uses the WCAG relative-luminance formula, so the resulting
//! pairings are guaranteed to be on the more-readable side of the available choices.
//!
//! The primary entry points are [`pick_ink`] (single color) and [`resolve_ink_colors`] (all
//! semantic tokens in a [`Spec`](crate::Spec)).

use std::collections::BTreeMap;

use crate::error::Error;
use crate::spec::SemanticValue;

/// Compute WCAG relative luminance from sRGB components in 0..=255.
#[must_use]
pub(crate) fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    fn linearize(c: u8) -> f64 {
        let s = f64::from(c) / 255.0;
        if s <= 0.040_45 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// WCAG contrast ratio between two relative-luminance values.
///
/// Returns a value in the range `1.0..=21.0` (black on white is `21.0`).
#[must_use]
pub(crate) fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

fn luminance_of(color: &str) -> Result<f64, Error> {
    let parsed = csscolorparser::parse(color).map_err(|source| Error::InvalidColor {
        color: color.to_string(),
        source,
    })?;
    let [r, g, b, _] = parsed.to_rgba8();
    Ok(relative_luminance(r, g, b))
}

/// Given a background color and a pair of candidate ink colors, return the higher-contrast ink.
///
/// All three colors must be parseable by [`csscolorparser`].
///
/// # Errors
///
/// Returns [`Error::InvalidColor`] if any of the input colors cannot be parsed.
///
/// [`Error::InvalidColor`]: crate::Error::InvalidColor
pub(crate) fn pick_ink(background: &str, light_ink: &str, dark_ink: &str) -> Result<String, Error> {
    let bg_lum = luminance_of(background)?;
    let light_lum = luminance_of(light_ink)?;
    let dark_lum = luminance_of(dark_ink)?;

    if contrast_ratio(bg_lum, light_lum) >= contrast_ratio(bg_lum, dark_lum) {
        Ok(light_ink.to_string())
    } else {
        Ok(dark_ink.to_string())
    }
}

/// Resolve ink colors for every semantic token in a spec.
///
/// For each `SemanticValue::PaletteRef(palette, shade)`, the corresponding palette/shade is
/// looked up in `resolved`, contrast-checked against `light_ink` / `dark_ink`, and the better
/// ink is recorded. For `SemanticValue::Raw` values that start with `#`, the same logic applies;
/// non-hex raw values are skipped.
///
/// # Errors
///
/// Returns [`Error::UnknownPalette`] if a semantic token references a palette that doesn't exist.
/// Returns [`Error::UnknownShade`] if a semantic token references a shade that doesn't exist.
/// Returns [`Error::InvalidColor`] if a background or ink color cannot be parsed.
///
/// [`Error::UnknownPalette`]: crate::Error::UnknownPalette
/// [`Error::UnknownShade`]: crate::Error::UnknownShade
/// [`Error::InvalidColor`]: crate::Error::InvalidColor
pub(crate) fn resolve_ink_colors(
    semantic: &BTreeMap<String, SemanticValue>,
    resolved: &BTreeMap<String, Vec<String>>,
    shades: &[u32],
    light_ink: &str,
    dark_ink: &str,
) -> Result<BTreeMap<String, String>, Error> {
    let mut out = BTreeMap::new();
    for (name, value) in semantic {
        match value {
            SemanticValue::PaletteRef(palette, shade) => {
                let palette_colors =
                    resolved.get(palette).ok_or_else(|| Error::UnknownPalette {
                        palette: palette.clone(),
                        context: format!(" in semantic token {name}"),
                    })?;
                let shade_idx =
                    shades
                        .iter()
                        .position(|s| s == shade)
                        .ok_or_else(|| Error::UnknownShade {
                            shade: *shade,
                            context: format!(" in semantic token {name}"),
                        })?;
                let hex = &palette_colors[shade_idx];
                let ink = pick_ink(hex, light_ink, dark_ink)?;
                out.insert(name.clone(), ink);
            }
            SemanticValue::Raw(raw) => {
                if raw.starts_with('#') {
                    let ink = pick_ink(raw, light_ink, dark_ink)?;
                    out.insert(name.clone(), ink);
                }
                // Non-hex raw values are skipped — we can't determine their luminance.
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_black_is_zero() {
        let l = relative_luminance(0, 0, 0);
        assert!((l - 0.0).abs() < 0.001);
    }

    #[test]
    fn luminance_white_is_one() {
        let l = relative_luminance(255, 255, 255);
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let ratio = contrast_ratio(
            relative_luminance(0, 0, 0),
            relative_luminance(255, 255, 255),
        );
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn pick_ink_on_dark_background_returns_light() {
        let ink = pick_ink("#0a0a0a", "#ffffff", "#000000").unwrap();
        assert_eq!(ink, "#ffffff");
    }

    #[test]
    fn pick_ink_on_light_background_returns_dark() {
        let ink = pick_ink("#f0f0f0", "#ffffff", "#000000").unwrap();
        assert_eq!(ink, "#000000");
    }

    #[test]
    fn pick_ink_on_mid_teal_returns_dark() {
        let ink = pick_ink("#31d4b8", "#e8edf4", "#080b12").unwrap();
        assert_eq!(ink, "#080b12");
    }

    #[test]
    fn pick_ink_on_dark_neutral_returns_light() {
        let ink = pick_ink("#080b12", "#e8edf4", "#080b12").unwrap();
        assert_eq!(ink, "#e8edf4");
    }

    #[test]
    fn pick_ink_rejects_invalid_color() {
        let err = pick_ink("not-a-color", "#ffffff", "#000000").unwrap_err();
        assert!(matches!(err, Error::InvalidColor { .. }));
    }

    fn test_resolved() -> BTreeMap<String, Vec<String>> {
        let mut m = BTreeMap::new();
        m.insert(
            "primary".into(),
            vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()],
        );
        m
    }

    #[test]
    fn resolve_ink_colors_produces_entry_per_semantic() {
        let mut semantic = BTreeMap::new();
        semantic.insert(
            "bg".into(),
            SemanticValue::PaletteRef("primary".into(), 100),
        );
        semantic.insert(
            "text".into(),
            SemanticValue::PaletteRef("primary".into(), 300),
        );
        let inks = resolve_ink_colors(
            &semantic,
            &test_resolved(),
            &[100, 200, 300],
            "#ffffff",
            "#000000",
        )
        .unwrap();
        assert_eq!(inks.len(), 2);
        assert!(inks.contains_key("bg"));
        assert!(inks.contains_key("text"));
    }

    #[test]
    fn resolve_ink_colors_unknown_palette_errors() {
        let mut semantic = BTreeMap::new();
        semantic.insert("bg".into(), SemanticValue::PaletteRef("nope".into(), 100));
        let err = resolve_ink_colors(
            &semantic,
            &test_resolved(),
            &[100, 200, 300],
            "#ffffff",
            "#000000",
        )
        .unwrap_err();
        assert!(matches!(err, Error::UnknownPalette { .. }));
    }

    #[test]
    fn resolve_ink_colors_skips_non_hex_raw() {
        let mut semantic = BTreeMap::new();
        semantic.insert("muted".into(), SemanticValue::Raw("rgba(0,0,0,0.5)".into()));
        let inks = resolve_ink_colors(
            &semantic,
            &test_resolved(),
            &[100, 200, 300],
            "#ffffff",
            "#000000",
        )
        .unwrap();
        assert!(inks.is_empty());
    }

    #[test]
    fn resolve_ink_colors_handles_hex_raw() {
        let mut semantic = BTreeMap::new();
        semantic.insert("muted".into(), SemanticValue::Raw("#0a0a0a".into()));
        let inks = resolve_ink_colors(
            &semantic,
            &test_resolved(),
            &[100, 200, 300],
            "#ffffff",
            "#000000",
        )
        .unwrap();
        assert_eq!(inks.get("muted").map(String::as_str), Some("#ffffff"));
    }
}
