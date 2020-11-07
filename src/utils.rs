use i18n_embed::WebLanguageRequester;
use std::sync::Once;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn select_language() {
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        let requested_languages = WebLanguageRequester::requested_languages();
        age::localizer().select(&requested_languages).unwrap();
    });
}
