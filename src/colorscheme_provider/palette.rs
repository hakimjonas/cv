use serde::{Deserialize, Serialize};

/// Universal color palette that all providers convert to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Base colors (0-7)
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    // Bright colors (8-15)
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,

    // Special colors
    pub background: String,
    pub foreground: String,
    pub cursor: Option<String>,
    pub selection: Option<String>,
}

/// Trait for converting color palettes to CSS
pub trait ToCss {
    fn to_css_variables(&self) -> String;
    fn to_theme_css(&self, theme_class: &str) -> String;
}

impl ToCss for ColorPalette {
    fn to_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");

        // Map terminal colors to CSS variables
        css.push_str(&format!("  --color-background: {};\n", self.background));
        css.push_str(&format!("  --color-text: {};\n", self.foreground));
        css.push_str(&format!("  --color-primary: {};\n", self.blue));
        css.push_str(&format!("  --color-secondary: {};\n", self.cyan));
        css.push_str(&format!("  --color-accent: {};\n", self.magenta));
        css.push_str(&format!("  --color-error: {};\n", self.red));
        css.push_str(&format!("  --color-warning: {};\n", self.yellow));
        css.push_str(&format!("  --color-success: {};\n", self.green));

        // Surface colors
        css.push_str(&format!("  --color-surface: {};\n", self.bright_black));
        css.push_str(&format!("  --color-overlay: {};\n", self.black));
        css.push_str(&format!("  --color-muted: {};\n", self.bright_black));
        css.push_str(&format!("  --color-subtle: {};\n", self.white));

        // Additional mappings
        css.push_str(&format!("  --color-border: {};\n", self.bright_black));
        css.push_str(&format!("  --color-text-light: {};\n", self.white));
        css.push_str(&format!("  --color-background-light: {};\n", self.black));
        css.push_str(&format!(
            "  --color-card-background: {};\n",
            self.background
        ));

        if let Some(cursor) = &self.cursor {
            css.push_str(&format!("  --color-cursor: {};\n", cursor));
        }
        if let Some(selection) = &self.selection {
            css.push_str(&format!("  --color-selection: {};\n", selection));
        }

        css.push_str("}\n");
        css
    }

    fn to_theme_css(&self, theme_class: &str) -> String {
        format!(
            "{} {{\n  /* Generated from terminal color scheme */\n{}}}\n",
            theme_class,
            self.to_css_variables()
                .lines()
                .skip(1) // Skip ":root {"
                .take_while(|line| !line.contains('}'))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
