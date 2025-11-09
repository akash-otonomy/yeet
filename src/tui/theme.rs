use ratatui::style::Color;

/// Retro 8-bit color palette inspired by classic arcade games and DOS terminals
pub struct RetroTheme;

impl RetroTheme {
    // Primary Colors - Vibrant and punchy (used in YEET logo)
    pub const CYAN: Color = Color::Rgb(0, 255, 255);        // Electric cyan
    pub const MAGENTA: Color = Color::Rgb(255, 0, 255);     // Hot pink magenta
    pub const YELLOW: Color = Color::Rgb(255, 255, 0);      // Bright yellow
    pub const GREEN: Color = Color::Rgb(0, 255, 159);       // Neon green
    pub const DARK_GRAY: Color = Color::Rgb(64, 64, 64);    // Dark gray
}
