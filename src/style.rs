use gtk4::CssProvider;
use gtk4::gdk::Display;

const GREETER_CSS: &str = "
/* ── Sfondo fullscreen ─────────────────────────────────────────── */
.greeter-root {
    background-color: transparent;
}

.greeter-wallpaper {
    background-color: #131318;
}

.greeter-overlay {
    background-color: alpha(black, 0.35);
}

/* ── Clock ──────────────────────────────────────────────────────── */
.clock-time {
    color: white;
    font-size: 64px;
    font-family: 'Nunito', sans-serif;
    font-weight: bold;
    text-shadow: 0 2px 8px alpha(black, 0.5);
}

.clock-date {
    color: alpha(white, 0.8);
    font-size: 18px;
    font-family: 'Nunito', sans-serif;
    font-weight: 500;
    text-shadow: 0 1px 4px alpha(black, 0.5);
}

/* ── Login card ─────────────────────────────────────────────────── */
.login-card {
    background-color: #1b1b21;
    border-radius: 13px;
    border: 1px solid alpha(#e4e1e9, 0.1);
    padding: 20px 20px 6px 20px;
    min-width: 360px;
}

/* ── Avatar circolare ───────────────────────────────────────────── */
.avatar-frame {
    border-radius: 50%;
    min-width: 72px;
    min-height: 72px;
    background-color: #2a292f;
}

.avatar-frame image {
    border-radius: 50%;
}

/* ── Username label ─────────────────────────────────────────────── */
.username-label {
    color: #e4e1e9;
    font-size: 15px;
    font-family: 'Nunito', sans-serif;
    font-weight: 700;
    margin-top: 4px;
}

/* ── User picker ────────────────────────────────────────────────── */
.user-dropdown {
    background-color: #2a292f;
    border-radius: 13px;
    color: #e4e1e9;
    font-family: 'Nunito', sans-serif;
    font-size: 14px;
    margin-top: 8px;
}

/* ── Password entry e session dropdown: identici ────────────────── */
.password-entry,
.session-dropdown {
    background-color: #2a292f;
    border-radius: 13px;
    color: #e4e1e9;
    font-size: 14px;
    font-family: 'Nunito', sans-serif;
    min-height: 36px;
    border: 1px solid alpha(#e4e1e9, 0.08);
}

.password-entry {
    padding: 2px 8px;
    margin-top: 0;
    transition: border-color 200ms ease;
}

.password-entry:focus {
    border-color: #c2c1ff;
}

.session-dropdown {
    margin-top: 0;
}

/* ── Bottone Login ──────────────────────────────────────────────── */
.login-button {
    background-color: #c2c1ff;
    color: #2a2a60;
    border-radius: 13px;
    padding: 0px 16px;
    font-family: 'Nunito', sans-serif;
    font-size: 14px;
    font-weight: 600;
    border: none;
    transition: background-color 200ms ease;
    min-height: 36px;
}

.login-button:hover {
    background-color: mix(#c2c1ff, white, 0.85);
}

.login-button:active {
    background-color: mix(#c2c1ff, black, 0.85);
}

.login-button-icon {
    font-family: 'Material Symbols Outlined';
    font-size: 18px;
    color: #2a2a60;
}

/* ── Errore ─────────────────────────────────────────────────────── */
.error-label {
    color: #ffb4ab;
    font-size: 12px;
    font-family: 'Nunito', sans-serif;
    font-weight: 500;
}

/* ── Icone Material Symbols nel session picker ──────────────────── */
.session-icon {
    font-family: 'Material Symbols Outlined';
    font-size: 18px;
    color: #e4e1e9;
}

.session-name {
    color: #e4e1e9;
    font-family: 'Nunito', sans-serif;
    font-size: 14px;
}

/* ── OSK placeholder ────────────────────────────────────────────── */
.osk-placeholder {
    min-height: 0px;
    background-color: transparent;
}

/* ── Monitor secondari ──────────────────────────────────────────── */
.secondary-clock-time {
    color: white;
    font-size: 80px;
    font-family: 'Nunito', sans-serif;
    font-weight: bold;
    text-shadow: 0 2px 8px alpha(black, 0.5);
}

.secondary-clock-date {
    color: alpha(white, 0.8);
    font-size: 22px;
    font-family: 'Nunito', sans-serif;
    font-weight: 500;
    text-shadow: 0 1px 4px alpha(black, 0.5);
}
";

pub fn load(display: &Display) {
    let provider = CssProvider::new();
    provider.load_from_data(GREETER_CSS);
    gtk4::style_context_add_provider_for_display(
        display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );
}
