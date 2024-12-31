use tui::style::{Color, Style, Modifier};

#[derive(Clone, Copy)]
pub enum Theme {
    Rust,
    Zinc,
}

impl Theme {
    pub fn style(&self) -> ThemeStyle {
        match self {
            Theme::Rust => ThemeStyle {
                title_color: Color::Rgb(183, 65, 14),
                border_color: Color::Rgb(139, 69, 19),
                text_color: Color::Rgb(255, 160, 122),
                error_color: Color::Red,
                selected_bg_color: Color::Rgb(205, 92, 92),
            },
            Theme::Zinc => ThemeStyle {
                title_color: Color::Rgb(192, 192, 192), // Silver
                border_color: Color::Rgb(50, 50, 50),     // Dark Gray
                text_color: Color::Rgb(220, 220, 220),   // Light Gray
                error_color: Color::Red,
                selected_bg_color: Color::Rgb(100, 100, 100), // Darker Gray
            },
        }
    }
}

pub struct ThemeStyle {
    pub title_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub error_color: Color,
    pub selected_bg_color: Color,
}
