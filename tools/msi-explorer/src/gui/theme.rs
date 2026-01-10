//! Modern theme for MSI Explorer

#![allow(dead_code)]

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2, Visuals};

/// Modern dark theme colors
pub struct Theme;

impl Theme {
    // Background colors
    pub const BG_DARK: Color32 = Color32::from_rgb(24, 24, 27);      // zinc-900
    pub const BG_MEDIUM: Color32 = Color32::from_rgb(39, 39, 42);    // zinc-800
    pub const BG_LIGHT: Color32 = Color32::from_rgb(63, 63, 70);     // zinc-700
    pub const BG_HOVER: Color32 = Color32::from_rgb(82, 82, 91);     // zinc-600

    // Accent colors
    pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);     // blue-500
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // blue-400
    pub const ACCENT_MUTED: Color32 = Color32::from_rgb(30, 58, 138); // blue-900

    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 250);   // zinc-50
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(161, 161, 170); // zinc-400
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);     // zinc-500

    // Status colors
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);     // green-500
    pub const WARNING: Color32 = Color32::from_rgb(234, 179, 8);     // yellow-500
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);       // red-500

    // Border
    pub const BORDER: Color32 = Color32::from_rgb(63, 63, 70);       // zinc-700
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(82, 82, 91); // zinc-600

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        // Window
        visuals.window_fill = Self::BG_MEDIUM;
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.window_rounding = Rounding::same(8.0);

        // Panel
        visuals.panel_fill = Self::BG_DARK;

        // Widgets
        visuals.widgets.noninteractive.bg_fill = Self::BG_MEDIUM;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        visuals.widgets.inactive.bg_fill = Self::BG_LIGHT;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);

        visuals.widgets.active.bg_fill = Self::ACCENT;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.active.rounding = Rounding::same(6.0);

        visuals.widgets.open.bg_fill = Self::BG_HOVER;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.open.rounding = Rounding::same(6.0);

        // Selection
        visuals.selection.bg_fill = Self::ACCENT_MUTED;
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT);

        // Hyperlinks
        visuals.hyperlink_color = Self::ACCENT;

        // Extreme background
        visuals.extreme_bg_color = Self::BG_DARK;
        visuals.faint_bg_color = Self::BG_MEDIUM;

        // Striped table
        visuals.striped = true;

        ctx.set_visuals(visuals);

        // Set default spacing
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        ctx.set_style(style);
    }
}

/// Category icons (unicode)
pub fn category_icon(category: &str) -> &'static str {
    match category {
        "Core" => "◆",
        "File" => "◨",
        "Registry" => "◫",
        "UI" => "◧",
        "Custom Action" => "◉",
        "Service" => "◎",
        "Sequence" => "▤",
        "Validation" => "◈",
        _ => "○",
    }
}
