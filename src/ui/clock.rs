use chrono::Local;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

/// Crea un widget clock con time + date.
/// `large` = true per monitor secondari (font più grande, testo bianco diretto).
pub fn build(large: bool) -> Box {
    let container = Box::new(Orientation::Vertical, 4);
    container.set_halign(gtk4::Align::Center);

    let time_label = Label::new(None);
    let date_label = Label::new(None);

    if large {
        time_label.add_css_class("secondary-clock-time");
        date_label.add_css_class("secondary-clock-date");
    } else {
        time_label.add_css_class("clock-time");
        date_label.add_css_class("clock-date");
    }

    update_clock(&time_label, &date_label);

    container.append(&time_label);
    container.append(&date_label);

    // Aggiorna ogni secondo
    let tl = time_label.clone();
    let dl = date_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        update_clock(&tl, &dl);
        glib::ControlFlow::Continue
    });

    container
}

fn update_clock(time_label: &Label, date_label: &Label) {
    let now = Local::now();
    time_label.set_text(&now.format("%H:%M").to_string());
    date_label.set_text(&now.format("%A, %d %B %Y").to_string());
}
