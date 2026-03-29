use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub display_name: String,
    pub avatar_path: Option<PathBuf>,
}

pub fn list_users() -> Vec<UserInfo> {
    let mut users = Vec::new();

    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 7 {
                continue;
            }
            let uid: u32 = fields[2].parse().unwrap_or(0);
            if uid < 1000 || uid == 65534 {
                continue;
            }

            let shell = fields[6];
            // Escludi utenti senza shell interattiva (nix build users, system users)
            if shell.contains("nologin") || shell.contains("noshell") || shell.contains("false") {
                continue;
            }

            // Escludi utenti la cui home non esiste (nix build users hanno /var/empty o simili)
            let home = fields[5];
            if !Path::new(home).exists() {
                continue;
            }

            let username = fields[0].to_string();
            let gecos = fields[4];
            let display_name = gecos
                .split(',')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(&username)
                .to_string();

            let avatar_path = find_avatar(&username, home);

            users.push(UserInfo {
                username,
                display_name,
                avatar_path,
            });
        }
    }

    users
}

fn find_avatar(username: &str, home: &str) -> Option<PathBuf> {
    // Priorità: AccountsService → ~/.face.icon → ~/.face
    let accounts_path = PathBuf::from(format!("/var/lib/AccountsService/icons/{}", username));
    if accounts_path.exists() {
        return Some(accounts_path);
    }

    let home = PathBuf::from(home);

    let face_icon = home.join(".face.icon");
    if face_icon.exists() {
        return Some(face_icon);
    }

    let face = home.join(".face");
    if face.exists() {
        return Some(face);
    }

    None
}
