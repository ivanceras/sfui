#![recursion_limit = "256"]
//#![feature(trace_macros)]

pub use sauron;
pub use status::Status;
pub use theme::Theme;

//std::trace_macros!(true);

pub mod button;
pub mod frame;
mod status;
mod theme;

/// register all the components as custom element in the DOM
pub fn register_all() {
    button::register();
    //frame::register();
}
