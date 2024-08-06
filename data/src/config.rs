use std::fs;
use std::path::PathBuf;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use serde::Deserialize;
use thiserror::Error;

pub use self::buffer::Buffer;
pub use self::channel::Channel;
pub use self::file_transfer::FileTransfer;
pub use self::keys::Keyboard;
pub use self::notification::Notifications;
pub use self::proxy::Proxy;
pub use self::server::Server;
pub use self::sidebar::Sidebar;
use crate::audio::{self, Sound};
use crate::environment::config_dir;
use crate::server::Map as ServerMap;
use crate::theme::Palette;
use crate::{environment, Theme};

pub mod buffer;
pub mod channel;
pub mod file_transfer;
mod keys;
pub mod notification;
pub mod proxy;
pub mod server;
pub mod sidebar;

const CONFIG_TEMPLATE: &str = include_str!("../../config.toml");
const DEFAULT_THEME_FILE_NAME: &str = "ferra.toml";

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub themes: Themes,
    pub servers: ServerMap,
    pub proxy: Option<Proxy>,
    pub font: Font,
    pub scale_factor: ScaleFactor,
    pub buffer: Buffer,
    pub sidebar: Sidebar,
    pub keyboard: Keyboard,
    pub notifications: Notifications<Sound>,
    pub file_transfer: FileTransfer,
    pub tooltips: bool,
    pub pane_toggling: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ScaleFactor(f64);

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl From<f64> for ScaleFactor {
    fn from(value: f64) -> Self {
        ScaleFactor(value.clamp(0.1, 3.0))
    }
}

impl From<ScaleFactor> for f64 {
    fn from(value: ScaleFactor) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Font {
    pub family: Option<String>,
    pub size: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct Themes {
    pub default: Theme,
    pub all: Vec<Theme>,
}

impl Default for Themes {
    fn default() -> Self {
        Self {
            default: Theme::default(),
            all: vec![Theme::default()],
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let dir = environment::config_dir();

        if !dir.exists() {
            std::fs::create_dir_all(dir.as_path())
                .expect("expected permissions to create config folder");
        }

        dir
    }

    pub fn sounds_dir() -> PathBuf {
        let dir = Self::config_dir().join("sounds");

        if !dir.exists() {
            std::fs::create_dir_all(dir.as_path())
                .expect("expected permissions to create sounds folder");
        }

        dir
    }

    fn themes_dir() -> PathBuf {
        let dir = Self::config_dir().join("themes");

        if !dir.exists() {
            std::fs::create_dir_all(dir.as_path())
                .expect("expected permissions to create themes folder");
        }

        dir
    }

    fn path() -> PathBuf {
        Self::config_dir().join(environment::CONFIG_FILE_NAME)
    }

    pub fn load() -> Result<Self, Error> {
        #[derive(Deserialize)]
        pub struct Configuration {
            #[serde(default)]
            pub theme: String,
            pub servers: ServerMap,
            pub proxy: Option<Proxy>,
            #[serde(default)]
            pub font: Font,
            #[serde(default)]
            pub scale_factor: ScaleFactor,
            #[serde(default)]
            pub buffer: Buffer,
            #[serde(default)]
            pub sidebar: Sidebar,
            #[serde(default)]
            pub keyboard: Keyboard,
            #[serde(default)]
            pub notifications: Notifications,
            #[serde(default)]
            pub file_transfer: FileTransfer,
            #[serde(default = "default_tooltip")]
            pub tooltips: bool,
            #[serde(default = "default_pane_toggling")]
            pub pane_toggling: bool
        }

        let path = Self::path();
        let content = fs::read_to_string(path).map_err(|e| Error::Read(e.to_string()))?;

        let Configuration {
            theme,
            mut servers,
            font,
            proxy,
            scale_factor,
            buffer,
            sidebar,
            keyboard,
            notifications,
            file_transfer,
            tooltips,
            pane_toggling,
        } = toml::from_str(content.as_ref()).map_err(|e| Error::Parse(e.to_string()))?;

        servers.read_password_files()?;

        let loaded_notifications = notifications.load_sounds()?;

        let themes = Self::load_themes(&theme).unwrap_or_default();

        Ok(Config {
            themes,
            servers,
            font,
            proxy,
            scale_factor,
            buffer,
            sidebar,
            keyboard,
            notifications: loaded_notifications,
            file_transfer,
            tooltips,
            pane_toggling,
        })
    }

    fn load_themes(default_key: &str) -> Result<Themes, Error> {
        #[derive(Deserialize)]
        pub struct Data {
            #[serde(default)]
            pub name: String,
            #[serde(default)]
            pub palette: Palette,
        }

        let read_entry = |entry: fs::DirEntry| {
            let content = fs::read_to_string(entry.path())?;

            let Data { name, palette } =
                toml::from_str(content.as_ref()).map_err(|e| Error::Parse(e.to_string()))?;

            Ok::<Theme, Error>(Theme::new(name, &palette))
        };

        let mut all = vec![];
        let mut default = Theme::default();
        let mut has_halloy_theme = false;

        for entry in fs::read_dir(Self::themes_dir())? {
            let Ok(entry) = entry else {
                continue;
            };

            let Some(file_name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };

            if file_name.ends_with(".toml") {
                if let Ok(theme) = read_entry(entry) {
                    if file_name.strip_suffix(".toml").unwrap_or_default() == default_key
                        || file_name == default_key
                    {
                        default = theme.clone();
                    }
                    if file_name == DEFAULT_THEME_FILE_NAME {
                        has_halloy_theme = true;
                    }

                    all.push(theme);
                }
            }
        }

        if !has_halloy_theme {
            all.push(Theme::default());
        }

        Ok(Themes { default, all })
    }

    pub fn create_initial_config() {
        // Checks if a config file is there
        let config_file = Self::path();
        if config_file.exists() {
            return;
        }

        // Generate a unique nick
        let rand_nick = random_nickname();

        // Replace placeholder nick with unique nick
        let config_string = CONFIG_TEMPLATE.replace("__NICKNAME__", rand_nick.as_str());
        let config_bytes = config_string.as_bytes();

        // Create configuration path.
        let config_path = Self::config_dir().join("config.toml");

        let _ = fs::write(config_path, config_bytes);
    }
}

pub fn random_nickname() -> String {
    let mut rng = ChaCha8Rng::from_entropy();
    random_nickname_with_seed(&mut rng)
}

pub fn random_nickname_with_seed<R: Rng>(rng: &mut R) -> String {
    let rand_digit: u16 = rng.gen_range(1000..=9999);
    let rand_nick = format!("halloy{rand_digit}");

    rand_nick
}

pub fn create_themes_dir() {
    const CONTENT: &[u8] = include_bytes!("../../assets/themes/ferra.toml");

    // Create default theme file.
    let file = Config::themes_dir().join(DEFAULT_THEME_FILE_NAME);
    if !file.exists() {
        let _ = fs::write(file, CONTENT);
    }
}

/// Has YAML configuration file.
pub fn has_yaml_config() -> bool {
    config_dir().join("config.yaml").exists()
}

fn default_tooltip() -> bool {
    true
}

fn default_pane_toggling() -> bool {
    false
}

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("config could not be read: {0}")]
    Read(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Parse(String),
    #[error("error loading sound: {0}")]
    LoadSounds(#[from] audio::LoadError),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
