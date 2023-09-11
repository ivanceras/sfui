use sfui::frame::Frame;
use sfui::sauron;
use sfui::sauron::*;
use sfui::Theme;

struct App {
    frame: Frame<Msg>,
}

impl App {
    fn new() -> Self {
        let theme = Theme::black_on_white();
        Self {
            frame: Frame::default().with_theme(theme.clone()),
        }
    }
}

enum Msg {}

impl Application<Msg> for App {
    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        Cmd::none()
    }

    fn view(&self) -> Node<Msg> {
        node! {
            <sfui-frame>
                This is inside sfui-frame
                <button>This is a button</button>
            </sfui-frame>
        }
    }

    fn style(&self) -> Vec<String> {
        self.frame.style()
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
