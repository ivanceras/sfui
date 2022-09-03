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
            //theme: Theme::black_on_white(),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        node! {
            <div {style! {display:"flex"}}>
                 <sfui-button label="Hello..."
                    look="chipped"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::HelloClick/>
                 <sfui-button label="Regular"
                    look="regular"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::WorldClick/>
                 <sfui-button label="skewed"
                    look="skewed"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::WorldClick/>

                 <sfui-button label="Muted!"
                    look="muted"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::WorldClick/>

                 <sfui-button label="Disabled!"
                    look="disabled"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::WorldClick/>

                 <sfui-button label="Simple"
                    look="simple"
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    on_click=|_|Msg::WorldClick/>
            </div>
        }
    }

    fn style(&self) -> String {
        self.theme.style()
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
