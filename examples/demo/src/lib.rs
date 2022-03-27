use sfui::sauron;
use sfui::sauron::prelude::*;

struct App;

impl Application<()> for App {
    fn view(&self) -> Node<()> {
        node! {
            <p>
                "hello"
            </p>
        }
    }

    fn update(&mut self, _msg: ()) -> Cmd<Self, ()> {
        Cmd::none()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    Program::replace_body(App);
}
