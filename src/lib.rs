#![allow(warnings)]
#![recursion_limit = "256"]

pub use sauron;
pub use status::Status;
pub use theme::Theme;

pub mod button;
pub mod card;
pub mod dice;
pub mod frame;
mod status;
mod theme;

pub fn register_all() {
    button::register();
    frame::register();
    dice::register();
    card::register();
}
