mod app;
mod fuzzy_search;
mod generated;
mod models;
mod page;
mod util;
mod views;

use seed::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    seed::App::build(
        |url, orders| {
            orders.send_msg(app::routes(url));
            app::fetch_data(orders);
            app::Model::default()
        },
        app::update,
        app::view,
    )
    .window_events(app::window_events)
    .routes(app::routes)
    .finish()
    .run();

    Ok(())
}

// see cargo.toml for more info
cfg_if::cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

// see cargo.toml for more info
cfg_if::cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
