# tangible

Design tokens as data — colors, type, space, shadows, gradients, and contrast — rendered to CSS.

A library and command-line tool for turning a structured design-token specification (JSON, or any
[serde](https://serde.rs)-compatible format) into a CSS custom-property sheet you can drop into any
project.

## At a glance

```bash
# Install (once published)
cargo install tangible

# Render a spec file to CSS on stdout
tangible tokens.json

# Or to a file
tangible tokens.json -o dist/tokens.css
```

```rust
use tangible::{Renderer, Spec};

let spec: Spec = serde_json::from_str(json)?;
let css = Renderer::new().render(&spec)?;
std::fs::write("dist/tokens.css", css.to_string())?;
```

## What it generates

From a single spec file, `tangible` emits CSS custom properties for:

- **Palettes** — explicit hex lists or interpolated gradients (linear / catmull-rom / basis,
  blended in rgb / linear-rgb / oklab / lab).
- **Semantic aliases** — name your roles (`bg`, `surface`, `interactive`) and bind them to
  palette shades or raw values.
- **Ink (contrast)** — for every semantic color, `tangible` picks the higher-contrast ink
  (WCAG luminance) and emits both the ink color and a paired text-shadow.
- **Typography** — font families, sizes, weights, line-heights, letter-spacings.
- **Space and dimensions** — scale-aligned spacing and sizing tokens.
- **Borders** — radii and widths.
- **Shadows and glows** — multi-layer drop shadows and elevation glows.
- **Gradients** — both var-reference and pre-resolved (color-space-interpolated) gradient strings.
- **Transitions, z-index, opacity, overlay** — the remaining variables.

## Library or CLI

The `cli` feature is enabled by default. To use `tangible` as a library only:

```toml
[dependencies]
tangible = { version = "0.0.1", default-features = false }
```

## License

Dual-licensed under either of

- [MIT license](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.
