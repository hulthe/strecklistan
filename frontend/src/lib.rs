#![deny(unreachable_patterns)]

mod app;
mod fuzzy_search;
mod generated;
mod models;
mod notification_manager;
mod page;
mod util;
mod views;

use app::{Model, Msg};
use seed::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    seed::App::builder(app::update, app::view)
        .after_mount(after_mount)
        .window_events(app::window_events)
        .routes(app::routes)
        .build_and_start();

    Ok(())
}

fn after_mount(url: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    if let Some(msg) = app::routes(url) {
        orders.send_msg(msg);
    }
    app::fetch_data(orders);
    AfterMount::new(app::Model::default())
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
