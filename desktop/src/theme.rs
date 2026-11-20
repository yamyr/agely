//! Native values come from the existing shared design tokens.

use gpui::{Rgba, rgb};
use std::sync::LazyLock;

const CSS: &str = include_str!("../../src/ui/tokens.css");

pub struct Theme {
    pub background: Rgba,
    pub surface: Rgba,
    pub muted_surface: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub muted: Rgba,
    pub accent: Rgba,
    pub soft: Rgba,
    pub error_bg: Rgba,
    pub error_text: Rgba,
    pub body: f32,
    pub small: f32,
    pub title: f32,
    pub section: f32,
    pub focus_width: f32,
    pub control: f32,
    pub radius: f32,
    pub space_small: f32,
    pub space: f32,
    pub space_large: f32,
}

fn token(name: &str) -> &'static str {
    let marker = format!("--{name}:");
    CSS.split_once(&marker)
        .and_then(|(_, value)| value.split(';').next())
        .map(str::trim)
        .expect("a required shared design token is missing")
}

fn color(name: &str) -> Rgba {
    let value = token(name).trim_start_matches('#');
    rgb(u32::from_str_radix(value, 16).expect("a shared color token must contain a hex color"))
}

fn pixels(name: &str) -> f32 {
    let value = token(name);
    if let Some(value) = value.strip_suffix("px") {
        value.parse().expect("a shared size token must be numeric")
    } else {
        value
            .trim_end_matches("rem")
            .parse::<f32>()
            .expect("a shared type token must be in rem")
            * 16.0
    }
}

pub fn theme() -> &'static Theme {
    static THEME: LazyLock<Theme> = LazyLock::new(|| Theme {
        background: color("bg"),
        surface: color("surface"),
        muted_surface: color("surface-muted"),
        border: color("border"),
        text: color("text"),
        muted: color("text-muted"),
        accent: color("color-accent-strong"),
        soft: color("color-accent-soft"),
        error_bg: color("band-limited-soft"),
        error_text: color("band-limited-ink"),
        body: pixels("text-sm"),
        small: pixels("text-xs"),
        title: pixels("text-2xl"),
        section: pixels("text-lg"),
        focus_width: pixels("focus-ring-width"),
        control: pixels("control-h"),
        radius: pixels("radius-md"),
        space_small: pixels("space-2"),
        space: pixels("space-4"),
        space_large: pixels("space-5"),
    });
    &THEME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_tokens_keep_accessible_text_and_control_sizes() {
        let theme = theme();
        assert!(theme.small >= 16.0);
        assert!(theme.body >= theme.small);
        assert!(theme.control >= 56.0);
    }
}
