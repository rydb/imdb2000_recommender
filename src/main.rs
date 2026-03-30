#![forbid(unstable_features)]
#![recursion_limit = "256"]

use std::env;

use crate::ui::app;
mod charts;
mod ui;

/// On wayland, text scaling is sometimes inconsistent due to some environment variables not being set.
/// This should fix that.
pub fn linux_env_vars() {
    // SAFETY: set_var is safe inside of single_threaded environments
    unsafe {
        env::set_var("GDK_BACKEND", "x11");
        env::set_var("GDK_SCALE", "1");
        env::set_var("GDK_DPI_SCALE", "1");
        env::set_var("WINIT_X11_SCALE_FACTOR", "1");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    linux_env_vars();

    dioxus::launch(app);
}
