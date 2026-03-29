#[derive(Debug, Clone)]
pub struct Session {
    pub name: &'static str,
    pub cmd: Vec<String>,
    pub icon: &'static str,
}

pub fn discover_sessions(_dirs: &[String]) -> Vec<Session> {
    vec![
        Session {
            name: "Evie",
            cmd: vec!["start-hyprland".into()],
            icon: "splitscreen_left",
        },
        Session {
            name: "TTY",
            cmd: vec!["/run/current-system/sw/bin/bash".into(), "--login".into()],
            icon: "terminal",
        },
    ]
}
