pub mod theme;

use crate::tui::theme::RetroTheme;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};
use throbber_widgets_tui::ThrobberState;

pub struct YeetTui {
    pub throbber_state: ThrobberState,
    pub frame_count: u32,
}

impl YeetTui {
    pub fn new() -> Self {
        Self {
            throbber_state: ThrobberState::default(),
            frame_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.throbber_state.calc_next();
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    /// Render the colorful YEET logo with gradient effect
    pub fn render_logo(&self, frame: &mut Frame, area: Rect) {
        // Colorful YEET ASCII art - each letter gets different color
        let logo = vec![
            Line::from(vec![
                "  ██╗   ██╗".fg(RetroTheme::MAGENTA),
                "███████╗".fg(RetroTheme::CYAN),
                "███████╗".fg(RetroTheme::YELLOW),
                "████████╗".fg(RetroTheme::GREEN),
            ]),
            Line::from(vec![
                "  ╚██╗ ██╔╝".fg(RetroTheme::MAGENTA),
                "██╔════╝".fg(RetroTheme::CYAN),
                "██╔════╝".fg(RetroTheme::YELLOW),
                "╚══██╔══╝".fg(RetroTheme::GREEN),
            ]),
            Line::from(vec![
                "   ╚████╔╝ ".fg(RetroTheme::MAGENTA),
                "█████╗  ".fg(RetroTheme::CYAN),
                "█████╗  ".fg(RetroTheme::YELLOW),
                "   ██║   ".fg(RetroTheme::GREEN),
                "   v0.1.0".fg(RetroTheme::DARK_GRAY),
            ]),
            Line::from(vec![
                "    ╚██╔╝  ".fg(RetroTheme::MAGENTA),
                "██╔══╝  ".fg(RetroTheme::CYAN),
                "██╔══╝  ".fg(RetroTheme::YELLOW),
                "   ██║   ".fg(RetroTheme::GREEN),
            ]),
            Line::from(vec![
                "     ██║   ".fg(RetroTheme::MAGENTA),
                "███████╗".fg(RetroTheme::CYAN),
                "███████╗".fg(RetroTheme::YELLOW),
                "   ██║   ".fg(RetroTheme::GREEN),
            ]),
            Line::from(vec![
                "     ╚═╝   ".fg(RetroTheme::MAGENTA),
                "╚══════╝".fg(RetroTheme::CYAN),
                "╚══════╝".fg(RetroTheme::YELLOW),
                "   ╚═╝   ".fg(RetroTheme::GREEN),
            ]),
        ];

        frame.render_widget(Paragraph::new(logo).centered(), area);
    }
}

impl Default for YeetTui {
    fn default() -> Self {
        Self::new()
    }
}
