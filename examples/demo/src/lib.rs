#![allow(warnings)]
use sfui::button::{self, Button};
use sfui::dice::{self, Dice};
use sfui::frame::{self, Frame};
use sfui::sauron;
use sfui::sauron::{
    html::{attributes::*, *},
    *,
};
use sfui::Theme;

enum Msg {
    ButtonMsg(button::Msg),
    FrameMsg(Box<frame::Msg<Msg>>),
    BtnFrameMsg(Box<frame::Msg<Msg>>),
    DiceMsg(Box<dice::Msg<Msg>>),
    HelloClick,
    InStructButtonClick,
}

struct App {
    theme: Theme,
    button: Button<Msg>,
    frame: Frame<Msg>,
    btn_frame: Frame<Msg>,
    dice: Dice<Msg>,
}

impl App {
    fn new() -> Self {
        let theme = Theme::black_on_white();
        let mut button = Button::with_label("This is a long label with some other labels")
            .with_theme(theme.clone());
        button.add_click_listener(|_me| Msg::InStructButtonClick);

        App {
            theme: theme.clone(),
            button,
            frame: Frame::default().with_theme(theme.clone()),
            btn_frame: Frame::default().with_theme(theme.clone()),
            dice: Dice::new(),
        }
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        Cmd::none()
    }

    fn view(&self) -> Node<Msg> {
        let label = "The quick brown fox jumps over the lazy dog!";
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
        //let features = ["chipped"];
        //let statuses = ["none"];

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
                <div>
                /*
                    {self.frame.view([img(
                        [
                            src("./assets/moon.jpg"),
                            style! {display:"block", width: px(500)},
                        ],
                        [],
                    )]).map_msg(|fmsg|Msg::FrameMsg(Box::new(fmsg)))}
                */
                </div>
                <div>
                /*
                    {self.btn_frame.view([
                        self.dice.view([
                            img([src("assets/moon.jpg"),
                                style! {display:"block", width: px(500), height: px(500)}
                            ],[])
                            ])
                        .map_msg(|dmsg|Msg::DiceMsg(Box::new(dmsg)))
                     ]).map_msg(|fmsg|Msg::BtnFrameMsg(Box::new(fmsg)))}
                */
                </div>
                <sfui-frame
                    theme-primary=&self.theme.primary_color
                    theme-background=&self.theme.background_color>
                        <span>"This are the content of sfui-frame!!"</span>
                        <div>"This ia kid in a div"</div>
                </sfui-frame>
                <div class="dice-container">
                    {self.dice.view([
                            img([src("assets/moon.jpg"),
                                style! {display:"block", width: px(500), height: px(500)}
                            ],[])
                            /*
                        {self.frame.view([img(
                            [
                                src("./assets/moon.jpg"),
                                style! {display:"block", width: px(500), height: px(500)},
                            ],
                            [],
                        )]).map_msg(|fmsg|Msg::FrameMsg(Box::new(fmsg)))}
                        */
                        ])
                        .map_msg(|dmsg|Msg::DiceMsg(Box::new(dmsg))) }
                </div>
            </div>
        }
    }

    fn style(&self) -> Vec<String> {
        [
            self.theme.style(),
            self.button.style(),
            self.frame.style(),
            self.dice.style(),
        ]
        .concat()
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
            Msg::BtnFrameMsg(fmsg) => {
                let effects = self.btn_frame.update(*fmsg);
                Cmd::from(effects.localize(|fmsg| Msg::FrameMsg(Box::new(fmsg))))
            }
            Msg::DiceMsg(dmsg) => {
                let effects = self
                    .dice
                    .update(*dmsg)
                    .localize(|dmsg| Msg::DiceMsg(Box::new(dmsg)));
                Cmd::from(effects)
            }
            Msg::InStructButtonClick => {
                log::info!(
                    "You clicked on the button instance that is attached to the App struct!"
                );
                Cmd::none()
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    console_error_panic_hook::set_once();

    let container = sauron::document()
        .query_selector(".container")
        .ok()
        .flatten()
        .expect("must have the .container in index.html");
    Program::append_to_mount(App::new(), &container);
}
