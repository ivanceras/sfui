use sfui::card::Card;
use sfui::frame::Frame;
use sfui::sauron;
use sfui::sauron::*;
use sfui::Theme;

struct App {
    theme: Theme,
    frame: Frame<Msg>,
    card: Card<Msg>,
}

impl App {
    fn new() -> Self {
        let theme = Theme::green_on_black();
        Self {
            frame: Frame::default().with_theme(theme.clone()),
            card: Card::default().with_theme(theme.clone()),
            theme,
        }
    }
}

enum Msg {}

impl Application<Msg> for App {
    fn update(&mut self, _msg: Msg) -> Cmd<Self, Msg> {
        Cmd::none()
    }

    fn view(&self) -> Node<Msg> {
        node! {
            <sfui-frame theme-primary=&self.theme.primary_color theme-background=&self.theme.background_color>
                This is inside sfui-frame
                <button>This is a button</button>
                <br/>
                <p>
                    This is a paragraph
                </p>
            </sfui-frame>
            <sfui-card theme-primary=&self.theme.primary_color theme-background=&self.theme.background_color>
            </sfui-card>
        }
    }

    fn style(&self) -> Vec<String> {
        vec![self.frame.style(), self.theme.style()].concat()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();

    sfui::register_all();
    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::append_to_mount(App::new(), &container);
}
