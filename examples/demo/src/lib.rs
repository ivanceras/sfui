use sfui::button::{self, Button};
use sfui::frame::{self, Frame};
use sfui::sauron;
use sfui::sauron::prelude::*;
use sfui::Theme;

enum Msg {
    ButtonMsg(button::Msg),
    FrameMsg(Box<frame::Msg<Msg>>),
    HelloClick,
}

struct App {
    theme: Theme,
    button: Button<Msg>,
    frame: Frame<Msg>,
}

impl App {
    fn new() -> Self {
        //let theme = Theme::green_on_black();
        let theme = Theme::black_on_white();
        App {
            theme: theme.clone(),
            button: Button::with_label("This is a long label with some other labels")
                .with_theme(theme.clone()),
            frame: Frame::with_label("A frame")
                .with_theme(theme.clone())
                .with_content(img(
                    [
                        src("./assets/moon.jpg"),
                        style! {display:"block", width: px(500)},
                    ],
                    [],
                )),
        }
    }
}

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> {
        let label = "Hello!";
        let make_button = |label, feature, status| {
            node! {
                <div {style!{display:"block"}}>
                     <sfui-button label=label
                        feature=feature
                        status=status
                        theme-primary=&self.theme.primary_color
                        theme-background=&self.theme.background_color
                        on_click=|_|Msg::HelloClick/>
                </div>
            }
        };
        let features = [
            "chipped", "regular", "skewed", "muted", "disabled", "simple",
        ];
        let statuses = ["none", "success", "error", "warning", "info"];

        node! {
            <div>
                {self.button.view().map_msg(Msg::ButtonMsg)}
                <div {style!{display:"flex", flex_direction:"row"}}>
                    {
                        for feature in features{
                            node!{
                                <div {style!{display: "flex", flex_direction: "column"}}>
                                    {
                                        for status in statuses{
                                            make_button(label, feature, status)
                                        }
                                    }
                                </div>
                            }
                        }
                    }
                </div>
                {self.frame.view().map_msg(|fmsg|Msg::FrameMsg(Box::new(fmsg)))}
                <sfui-frame
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color
                    />
            </div>
        }
    }

    fn style(&self) -> String {
        [self.theme.style(), self.button.style(), self.frame.style()].join("\n")
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::HelloClick => {
                log::info!("Somebody clicked on the hello button..");
                Cmd::none()
            }
            Msg::ButtonMsg(bmsg) => {
                let effects = self.button.update(bmsg);
                Cmd::from(effects.localize(Msg::ButtonMsg))
            }
            Msg::FrameMsg(fmsg) => {
                let effects = self.frame.update(*fmsg);
                Cmd::from(effects.localize(|fmsg| Msg::FrameMsg(Box::new(fmsg))))
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    sfui::register_all();
    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::replace_mount(App::new(), &container);
}
