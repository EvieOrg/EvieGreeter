mod config;
mod greeter;
mod ipc;
mod session;
mod style;
mod ui;
mod users;

use config::Config;
use greeter::{Greeter, GreeterEvent};
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::gio::ApplicationFlags;
use std::sync::Arc;
use ui::login_card::LoginMsg;
fn main() {
    // Avvia runtime Tokio
    let rt = Arc::new(tokio::runtime::Runtime::new().expect("tokio runtime"));

    let app = Application::builder()
        .application_id("org.evie_desktop.greeter")
        .flags(ApplicationFlags::empty())
        .build();

    let rt_clone = rt.clone();
    app.connect_activate(move |app| {
        // Inizializza libadwaita (necessario per le variabili CSS @accent_bg_color ecc.)
        libadwaita::init().expect("failed to init libadwaita");

        // Carica config
        let config = Config::load();

        // Forza dark mode via libadwaita
        let adw_manager = libadwaita::StyleManager::default();
        adw_manager.set_color_scheme(libadwaita::ColorScheme::ForceDark);

        // Carica CSS
        if let Some(display) = Display::default() {
            style::load(&display);
        }

        // Scopri sessioni e utenti
        let sessions = session::discover_sessions(&config.session_dirs);
        let users = users::list_users();

        // Canali UI ↔ greeter state machine
        let (login_tx, login_rx) = async_channel::unbounded::<LoginMsg>();
        let (event_tx, event_rx) = async_channel::unbounded::<GreeterEvent>();

        // Ottieni monitor
        let display = Display::default().expect("no display");
        let monitors = display.monitors();
        let n = monitors.n_items();

        // Seleziona monitor primario per nome (da config) o fallback al primo
        let primary_monitor = if let Some(ref name) = config.primary_monitor {
            (0..n)
                .filter_map(|i| monitors.item(i).and_downcast::<gtk4::gdk::Monitor>())
                .find(|m| m.connector().map(|c| c.as_str() == name.as_str()).unwrap_or(false))
                .or_else(|| monitors.item(0).and_downcast::<gtk4::gdk::Monitor>())
                .expect("no monitor")
        } else {
            monitors.item(0).and_downcast::<gtk4::gdk::Monitor>().expect("no monitor")
        };

        let (_, card) = ui::window::build_primary(
            app,
            &primary_monitor,
            &users,
            sessions.clone(),
            &config,
            login_tx.clone(),
        );

        // Monitor secondari: tutti tranne il primario
        for i in 0..n {
            if let Some(monitor) = monitors.item(i).and_downcast::<gtk4::gdk::Monitor>() {
                if monitor.connector() != primary_monitor.connector() {
                    ui::window::build_secondary(app, &monitor, &config);
                }
            }
        }

        // Avvia greeter state machine in tokio
        let greeter = Greeter::new(login_rx, event_tx, sessions);
        rt_clone.spawn(async move {
            greeter.run().await;
        });

        // Gestisci eventi dal greeter → UI (sul main loop GTK)
        glib::spawn_future_local(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    GreeterEvent::ShowError(msg) => card.show_error(&msg),
                    GreeterEvent::ClearError => card.clear_error(),
                    GreeterEvent::ClearPassword => card.clear_password(),
                    GreeterEvent::SessionStarted => {
                        // Killa il processo padre (Hyprland) direttamente per uscita immediata
                        unsafe {
                            libc::kill(libc::getppid(), libc::SIGTERM);
                        }
                        std::process::exit(0);
                    }
                }
            }
        });
    });

    let _guard = rt.enter();
    app.run();
}
