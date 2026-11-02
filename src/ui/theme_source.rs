//! Where a theme comes from, and live reloading when that file changes

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::Theme;
use super::theme::hyprland_theme_paths;

/// How often the theme file is checked for changes
const CHECK_INTERVAL: Duration = Duration::from_secs(1);

/// The origin of a theme
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeSource {
    /// A theme compiled into ramwise (dark, light, ansi)
    Builtin(String),
    /// A TOML theme file (path may contain `~` or environment variables)
    Custom(String),
    /// The current Hyprland/Omarchy desktop theme. With `fallback_to_dark`,
    /// a missing colors file gives the dark theme instead of an error.
    Hyprland { fallback_to_dark: bool },
}

/// A theme file's path and contents
type ThemeFile = (PathBuf, String);

impl ThemeSource {
    /// Look a theme up by name. `custom` reads `custom_theme_file`.
    pub fn from_name(name: &str, custom_theme_file: &str) -> Result<Self, String> {
        match name.trim().to_lowercase().as_str() {
            builtin @ ("dark" | "light" | "ansi") => Ok(Self::Builtin(builtin.to_string())),
            "custom" => Ok(Self::Custom(custom_theme_file.to_string())),
            "hyprland" | "omarchy" => Ok(Self::Hyprland {
                fallback_to_dark: false,
            }),
            other => Err(format!(
                "unknown theme '{other}' (available: dark, light, ansi, hyprland, custom)"
            )),
        }
    }

    /// Read the file this theme is built from, if it has one and it exists
    fn read(&self) -> Option<ThemeFile> {
        match self {
            Self::Builtin(_) => None,
            Self::Custom(file) => {
                let path = PathBuf::from(shellexpand::full(file).ok()?.into_owned());
                let contents = fs::read_to_string(&path).ok()?;
                Some((path, contents))
            }
            Self::Hyprland { .. } => hyprland_theme_paths()
                .into_iter()
                .find_map(|path| fs::read_to_string(&path).ok().map(|c| (path, c))),
        }
    }

    /// Build the theme from an already read file (see `read`)
    fn build(&self, file: Option<&ThemeFile>) -> Result<Theme, String> {
        match (self, file) {
            (Self::Builtin(name), _) => match name.as_str() {
                "light" => Ok(Theme::light()),
                "ansi" => Ok(Theme::ansi4bit()),
                _ => Ok(Theme::dark()),
            },
            (Self::Custom(_), Some((path, contents))) => {
                Theme::from_toml(contents).map_err(|e| format!("{}: {e}", path.display()))
            }
            (Self::Custom(file), None) => Err(format!("{file} could not be read")),
            (Self::Hyprland { .. }, Some((path, contents))) => {
                Theme::from_hyprland_colors(contents)
                    .map_err(|e| format!("{}: {e}", path.display()))
            }
            (Self::Hyprland { fallback_to_dark }, None) => {
                if *fallback_to_dark {
                    Ok(Theme::dark())
                } else {
                    let searched: Vec<String> = hyprland_theme_paths()
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect();
                    Err(format!(
                        "no Hyprland theme colors found (searched: {})",
                        searched.join(", ")
                    ))
                }
            }
        }
    }

    /// Load the theme once
    pub fn load(&self) -> Result<Theme, String> {
        self.build(self.read().as_ref())
    }
}

/// Reloads a theme when the file it was built from changes
#[derive(Debug)]
pub struct ThemeWatcher {
    source: ThemeSource,
    last_file: Option<ThemeFile>,
    next_check: Instant,
}

impl ThemeWatcher {
    /// Load the theme and start watching its file
    pub fn new(source: ThemeSource) -> Result<(Self, Theme), String> {
        let file = source.read();
        let theme = source.build(file.as_ref())?;
        let watcher = Self {
            source,
            last_file: file,
            next_check: Instant::now() + CHECK_INTERVAL,
        };
        Ok((watcher, theme))
    }

    /// Check the theme file (at most once per second). Returns the reloaded
    /// theme, or the error if the changed file is invalid, when it changed.
    /// A file that disappears keeps the current theme until it comes back.
    pub fn poll(&mut self) -> Option<Result<Theme, String>> {
        if matches!(self.source, ThemeSource::Builtin(_)) {
            return None;
        }
        let now = Instant::now();
        if now < self.next_check {
            return None;
        }
        self.next_check = now + CHECK_INTERVAL;
        self.check()
    }

    /// Check the theme file right away
    fn check(&mut self) -> Option<Result<Theme, String>> {
        let file = self.source.read()?;
        if self.last_file.as_ref() == Some(&file) {
            return None;
        }
        let result = self.source.build(Some(&file));
        self.last_file = Some(file);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ramwise-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn from_name_knows_every_theme() {
        assert_eq!(
            ThemeSource::from_name(" Light ", "").unwrap(),
            ThemeSource::Builtin("light".into())
        );
        assert_eq!(
            ThemeSource::from_name("hyprland", "").unwrap(),
            ThemeSource::Hyprland {
                fallback_to_dark: false
            }
        );
        assert!(ThemeSource::from_name("unknown-theme", "").is_err());
    }

    #[test]
    fn builtin_themes_are_not_watched() {
        let (mut watcher, theme) = ThemeWatcher::new(ThemeSource::Builtin("light".into())).unwrap();
        assert_eq!(theme.bg, Theme::light().bg);
        watcher.next_check = Instant::now();
        assert!(watcher.poll().is_none());
    }

    #[test]
    fn custom_theme_reloads_when_the_file_changes() {
        let path = temp_file("reload.toml", "bg = [1, 2, 3]\n");
        let source = ThemeSource::Custom(path.display().to_string());
        let (mut watcher, theme) = ThemeWatcher::new(source).unwrap();
        assert_eq!(theme.bg, Color::Rgb(1, 2, 3));

        // Unchanged file: nothing to do
        assert!(watcher.check().is_none());

        // Changed file: new theme
        fs::write(&path, "bg = [4, 5, 6]\n").unwrap();
        let reloaded = watcher.check().unwrap().unwrap();
        assert_eq!(reloaded.bg, Color::Rgb(4, 5, 6));

        // Broken file: error, reported once
        fs::write(&path, "bg = \"nope\"\n").unwrap();
        assert!(watcher.check().unwrap().is_err());
        assert!(watcher.check().is_none());

        // Removed file: keep the current theme
        fs::remove_file(&path).unwrap();
        assert!(watcher.check().is_none());
        let _ = fs::remove_dir(path.parent().unwrap());
    }

    #[test]
    fn missing_custom_file_is_an_error() {
        let source = ThemeSource::Custom("/nonexistent/ramwise-theme.toml".into());
        assert!(ThemeWatcher::new(source).is_err());
    }
}
