use crate::config::Config;
use crate::session::Session;
use crate::ui::clock;
use crate::ui::login_card::{LoginCard, LoginMsg};
use crate::users::UserInfo;
use async_channel::Sender;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Orientation, Overlay, Picture};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

/// Finestra sul monitor primario: wallpaper + overlay + clock + login card
pub fn build_primary(
    app: &Application,
    monitor: &gtk4::gdk::Monitor,
    users: &[UserInfo],
    sessions: Vec<Session>,
    config: &Config,
    tx: Sender<LoginMsg>,
) -> (ApplicationWindow, LoginCard) {
    let win = ApplicationWindow::new(app);
    win.set_decorated(false);
    win.init_layer_shell();
    win.set_layer(Layer::Overlay);
    win.set_monitor(Some(monitor));
    win.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        win.set_anchor(edge, true);
    }
    win.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    let overlay = Overlay::new();
    overlay.add_css_class("greeter-root");

    // Wallpaper
    let wallpaper_box = Box::new(Orientation::Horizontal, 0);
    wallpaper_box.add_css_class("greeter-wallpaper");
    wallpaper_box.set_hexpand(true);
    wallpaper_box.set_vexpand(true);

    if let Some(ref wp_path) = config.wallpaper {
        let pic = Picture::for_filename(wp_path);
        pic.set_hexpand(true);
        pic.set_vexpand(true);
        wallpaper_box.append(&pic);
    }
    overlay.set_child(Some(&wallpaper_box));

    // Dim overlay
    let dim = Box::new(Orientation::Horizontal, 0);
    dim.add_css_class("greeter-overlay");
    dim.set_hexpand(true);
    dim.set_vexpand(true);
    overlay.add_overlay(&dim);

    // Clock: top-center, overlay indipendente
    let clock_widget = clock::build(false);
    clock_widget.set_halign(gtk4::Align::Center);
    clock_widget.set_valign(gtk4::Align::Start);
    clock_widget.set_margin_top(32);
    overlay.add_overlay(&clock_widget);

    // Login card: centrata geometricamente su X e Y
    let card = LoginCard::build(users, sessions, tx);
    card.widget.set_halign(gtk4::Align::Center);
    card.widget.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&card.widget);
    win.set_child(Some(&overlay));
    win.present();

    (win, card)
}

/// Finestra sui monitor secondari: solo wallpaper + clock
pub fn build_secondary(
    app: &Application,
    monitor: &gtk4::gdk::Monitor,
    config: &Config,
) -> ApplicationWindow {
    let win = ApplicationWindow::new(app);
    win.set_decorated(false);
    win.init_layer_shell();
    win.set_layer(Layer::Background);
    win.set_monitor(Some(monitor));
    win.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        win.set_anchor(edge, true);
    }

    let overlay = Overlay::new();

    let wallpaper_box = Box::new(Orientation::Horizontal, 0);
    wallpaper_box.add_css_class("greeter-wallpaper");
    wallpaper_box.set_hexpand(true);
    wallpaper_box.set_vexpand(true);

    if let Some(ref wp_path) = config.wallpaper {
        let pic = Picture::for_filename(wp_path);
        pic.set_hexpand(true);
        pic.set_vexpand(true);
        wallpaper_box.append(&pic);
    }
    overlay.set_child(Some(&wallpaper_box));

    let dim = Box::new(Orientation::Horizontal, 0);
    dim.add_css_class("greeter-overlay");
    dim.set_hexpand(true);
    dim.set_vexpand(true);
    overlay.add_overlay(&dim);

    let clock_widget = clock::build(true);
    clock_widget.set_halign(gtk4::Align::Center);
    clock_widget.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&clock_widget);

    win.set_child(Some(&overlay));
    win.present();
    win
}
