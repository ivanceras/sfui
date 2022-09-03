use sfui::button::{self, Button};
use sfui::sauron;
use sfui::sauron::prelude::*;
use sfui::Theme;

enum Msg {
    HelloClick,
    WorldClick,
}

struct App {
    theme: Theme,
}

impl App {
    fn new() -> Self {
        App {
            theme: Theme::green_on_black(),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        node! {
            <div {style! {display:"flex"}}>
                 <sfui-button label="Hello..." theme-primary="#0F0" theme-background="#000" on_click=|_|Msg::HelloClick/>
                 <sfui-button label="world!" theme-primary="#F00" theme-background="#000" on_click=|_|Msg::WorldClick/>
            </div>
        }
    }

    fn style(&self) -> String {
        let mut styles = self.theme.style();
        styles
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::HelloClick => {
                log::info!("Somebody clicked on the hello button..");
                Cmd::none()
            }
            Msg::WorldClick => {
                log::info!("Somebody clicked on the world! button..");
                Cmd::none()
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    button::register();
    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::replace_mount(App::new(), &container);
}
