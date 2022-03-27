use sfui::button::{self, Button};
use sfui::sauron;
use sfui::sauron::prelude::*;
use sfui::Theme;

enum Msg {
    ButtonMsg(button::Msg),
}

struct App {
    button: Button<Msg>,
    theme: Theme,
}

impl App {
    fn new() -> Self {
        App {
            button: Button::with_label("Hello").with_options(button::Options::full()),
            theme: Theme::green_on_black(),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        node! {
            <div>
                { self.button.view().map_msg(Msg::ButtonMsg) }
            </div>
        }
    }

    fn style(&self) -> String {
        let mut styles = self.theme.style();
        styles += &Button::<Msg>::style(&self.theme);
        styles
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::ButtonMsg(bmsg) => self.button.update(bmsg),
        };
        Cmd::none()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::replace_mount(App::new(), &container);
}
