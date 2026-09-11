mod window;
mod i18n;

use crate::window::CaffeineApplet;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    cosmic::applet::run::<CaffeineApplet>(())?;

    Ok(())
}
