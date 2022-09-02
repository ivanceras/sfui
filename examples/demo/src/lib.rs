use sfui::button::{self, Button};
use sfui::sauron;
use sfui::sauron::prelude::*;
use sfui::Theme;

enum Msg {
    ButtonChipdMsg(button::Msg),
    ButtonRegularMsg(button::Msg),
}

struct App {
    button_chipd: Button<Msg>,
    button_regular: Button<Msg>,
    theme: Theme,
}

impl App {
    fn new() -> Self {
        App {
            button_chipd: Button::with_label("I'm chipped")
                .with_options(button::Options::full())
                .chipped(),
            button_regular: Button::with_label("Regular button")
                .with_options(button::Options::regular()),
            theme: Theme::green_on_black(),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        div(
            [style! {display:"flex"}],
            [
                self.button_chipd.view().map_msg(Msg::ButtonChipdMsg),
                self.button_regular.view().map_msg(Msg::ButtonRegularMsg),
            ],
        )
    }

    fn style(&self) -> String {
        let mut styles = self.theme.style();
        styles += &Button::<Msg>::style(&self.theme);
        styles
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::ButtonChipdMsg(bmsg) => self.button_chipd.update(bmsg),
            Msg::ButtonRegularMsg(bmsg) => self.button_regular.update(bmsg),
        };
        Cmd::none()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::replace_mount(App::new(), &container);
}
