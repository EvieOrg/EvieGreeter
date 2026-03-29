use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub wallpaper: Option<String>,

    #[serde(default = "default_sessions_dirs")]
    pub session_dirs: Vec<String>,

    /// Nome del connettore del monitor primario (es. "DP-3", "HDMI-A-1").
    /// Se non specificato usa il primo monitor disponibile.
    #[serde(default)]
    pub primary_monitor: Option<String>,
}

fn default_sessions_dirs() -> Vec<String> {
    vec![
        "/run/current-system/sw/share/wayland-sessions".into(),
        "/run/current-system/sw/share/xsessions".into(),
        "/usr/share/wayland-sessions".into(),
        "/usr/share/xsessions".into(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallpaper: find_default_wallpaper(),
            session_dirs: default_sessions_dirs(),
            primary_monitor: None,
        }
    }
}

/// Cerca il wallpaper nel path dello share di Nix, poi nel path di config.
fn find_default_wallpaper() -> Option<String> {
    // Path installato da Nix (ricavato dal binario stesso)
    if let Ok(exe) = std::env::current_exe() {
        // $out/bin/evie-greeter → $out/share/evie-greeter/wallpaper
        if let Some(bin_dir) = exe.parent() {
            if let Some(out) = bin_dir.parent() {
                let wp = out.join("share/evie-greeter/wallpaper");
                if wp.exists() {
                    return Some(wp.to_string_lossy().into_owned());
                }
            }
        }
    }

    // Fallback: path di config manuale
    let config_wp = PathBuf::from("/etc/greeter/wallpaper");
    if config_wp.exists() {
        return Some(config_wp.to_string_lossy().into_owned());
    }

    None
}

impl Config {
    pub fn load() -> Self {
        let path = PathBuf::from("/etc/greeter/config.toml");
        if let Ok(content) = std::fs::read_to_string(&path) {
            let mut cfg: Config = toml::from_str(&content).unwrap_or_default();
            // Se il toml non specifica wallpaper, usa quello di default
            if cfg.wallpaper.is_none() {
                cfg.wallpaper = find_default_wallpaper();
            }
            cfg
        } else {
            Self::default()
        }
    }
}
