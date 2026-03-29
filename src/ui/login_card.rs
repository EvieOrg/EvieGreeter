use crate::session::Session;
use crate::users::UserInfo;
use async_channel::Sender;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DropDown, Image, Label, Orientation, PasswordEntry, StringList};
/// Messaggi inviati dalla UI al greeter state machine
#[derive(Debug)]
pub enum LoginMsg {
    UserSelected(String),
    PasswordSubmit(String),
    SessionSelected(usize),
}

#[allow(dead_code)]
pub struct LoginCard {
    pub widget: Box,
    pub error_label: Label,
    pub password_entry: PasswordEntry,
    pub session_dropdown: DropDown,
    pub sessions: Vec<Session>,
}

impl LoginCard {
    pub fn build(
        users: &[UserInfo],
        sessions: Vec<Session>,
        tx: Sender<LoginMsg>,
    ) -> Self {
        let card = Box::new(Orientation::Vertical, 6);
        card.add_css_class("login-card");
        card.set_halign(Align::Center);
        card.set_valign(Align::Center);

        // ── Avatar ──────────────────────────────────────────────────────
        let avatar = build_avatar(users.first());
        avatar.set_halign(Align::Center);
        card.append(&avatar);

        // ── Username (label o dropdown) ──────────────────────────────────
        let user_tx = tx.clone();
        let (username_widget, initial_user) = if users.len() <= 1 {
            let name = users
                .first()
                .map(|u| u.display_name.clone())
                .unwrap_or_else(|| "User".into());
            let label = Label::new(Some(&name));
            label.add_css_class("username-label");
            label.set_halign(Align::Center);
            let username = users
                .first()
                .map(|u| u.username.clone())
                .unwrap_or_default();
            (label.upcast::<gtk4::Widget>(), username)
        } else {
            let names: Vec<&str> = users.iter().map(|u| u.display_name.as_str()).collect();
            let model = StringList::new(&names);
            let dd = DropDown::new(Some(model), None::<gtk4::Expression>);
            dd.add_css_class("user-dropdown");
            let users_clone: Vec<UserInfo> = users.to_vec();
            dd.connect_selected_notify(move |d| {
                let idx = d.selected() as usize;
                if let Some(u) = users_clone.get(idx) {
                    let _ = user_tx.try_send(LoginMsg::UserSelected(u.username.clone()));
                }
            });
            let first_username = users.first().map(|u| u.username.clone()).unwrap_or_default();
            (dd.upcast::<gtk4::Widget>(), first_username)
        };
        card.append(&username_widget);

        // ── Password entry ───────────────────────────────────────────────
        let password_entry = PasswordEntry::new();
        password_entry.add_css_class("password-entry");
        password_entry.set_placeholder_text(Some("Password"));
        password_entry.set_show_peek_icon(true);

        let pw_tx = tx.clone();
        let pw_entry_clone = password_entry.clone();
        password_entry.connect_activate(move |_| {
            let pw = pw_entry_clone.text().to_string();
            let _ = pw_tx.try_send(LoginMsg::PasswordSubmit(pw));
        });
        card.append(&password_entry);

        // ── Error label ──────────────────────────────────────────────────
        let error_label = Label::new(None);
        error_label.add_css_class("error-label");
        error_label.set_halign(Align::Start);
        error_label.set_visible(false);
        card.append(&error_label);

        // ── Session picker ───────────────────────────────────────────────
        let session_model = gtk4::gio::ListStore::new::<gtk4::glib::BoxedAnyObject>();
        for s in &sessions {
            session_model.append(&gtk4::glib::BoxedAnyObject::new(s.clone()));
        }

        let factory = gtk4::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let row = Box::new(Orientation::Horizontal, 8);
            row.set_margin_start(4);
            row.set_margin_end(4);
            let icon = Label::new(None);
            icon.add_css_class("session-icon");
            let name = Label::new(None);
            name.add_css_class("session-name");
            row.append(&icon);
            row.append(&name);
            item.set_child(Some(&row));
        });
        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let obj = item.item().and_downcast::<gtk4::glib::BoxedAnyObject>().unwrap();
            let session = obj.borrow::<Session>();
            let row = item.child().and_downcast::<Box>().unwrap();
            let mut child = row.first_child();
            if let Some(icon_label) = child.as_ref().and_then(|w| w.downcast_ref::<Label>()) {
                icon_label.set_text(session.icon);
            }
            child = child.and_then(|w| w.next_sibling());
            if let Some(name_label) = child.as_ref().and_then(|w| w.downcast_ref::<Label>()) {
                name_label.set_text(session.name);
            }
        });

        let session_dropdown = DropDown::new(
            Some(session_model),
            None::<gtk4::Expression>,
        );
        session_dropdown.set_factory(Some(&factory));
        session_dropdown.set_list_factory(Some(&factory));
        session_dropdown.add_css_class("session-dropdown");
        let sess_tx = tx.clone();
        session_dropdown.connect_selected_notify(move |d| {
            let _ = sess_tx.try_send(LoginMsg::SessionSelected(d.selected() as usize));
        });
        card.append(&session_dropdown);

        // Invia sessione iniziale (idx 0)
        let _ = tx.try_send(LoginMsg::SessionSelected(0));

        // ── Bottoni ──────────────────────────────────────────────────────
        let icon_label = Label::new(Some("passkey"));
        icon_label.add_css_class("login-button-icon");

        let btn_label = Label::new(Some("Login"));

        let btn_content = Box::new(Orientation::Horizontal, 8);
        btn_content.set_halign(Align::Center);
        btn_content.append(&icon_label);
        btn_content.append(&btn_label);

        let login_btn = Button::new();
        login_btn.set_child(Some(&btn_content));
        login_btn.add_css_class("login-button");
        login_btn.set_halign(Align::Center);
        login_btn.set_margin_top(6);
        login_btn.set_margin_bottom(6);
        let btn_tx = tx.clone();
        let pw_for_btn = password_entry.clone();
        login_btn.connect_clicked(move |_| {
            let pw = pw_for_btn.text().to_string();
            let _ = btn_tx.try_send(LoginMsg::PasswordSubmit(pw));
        });
        card.append(&login_btn);

        // ── OSK placeholder ──────────────────────────────────────────────
        let osk_slot = Box::new(Orientation::Horizontal, 0);
        osk_slot.add_css_class("osk-placeholder");
        card.append(&osk_slot);

        // Invia l'utente iniziale
        let _ = tx.try_send(LoginMsg::UserSelected(initial_user));

        Self {
            widget: card,
            error_label,
            password_entry,
            session_dropdown,
            sessions,
        }
    }

    pub fn show_error(&self, msg: &str) {
        self.error_label.set_text(msg);
        self.error_label.set_visible(true);
    }

    pub fn clear_error(&self) {
        self.error_label.set_text("");
        self.error_label.set_visible(false);
    }

    pub fn clear_password(&self) {
        self.password_entry.set_text("");
    }
}

fn build_avatar(user: Option<&UserInfo>) -> gtk4::Widget {
    let img = if let Some(u) = user {
        if let Some(ref path) = u.avatar_path {
            // Verifica che il file sia effettivamente leggibile dal processo corrente
            if std::fs::File::open(path).is_ok() {
                Image::from_file(path)
            } else {
                Image::from_icon_name("avatar-default-symbolic")
            }
        } else {
            Image::from_icon_name("avatar-default-symbolic")
        }
    } else {
        Image::from_icon_name("avatar-default-symbolic")
    };

    img.set_pixel_size(72);
    img.set_overflow(gtk4::Overflow::Hidden);
    img.add_css_class("avatar-frame");
    img.upcast()
}
