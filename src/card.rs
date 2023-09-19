use crate::frame::{self, Frame};
use crate::Theme;
use sauron::{
    dom::{Callback, WebComponent},
    html::{attributes::*, events::*, *},
    *,
};

pub enum Msg<XMSG> {
    External(XMSG),
    FrameMsg(frame::Msg<XMSG>),
}

pub struct Card<XMSG> {
    theme: Theme,
    children: Vec<Node<XMSG>>,
    frame: Frame<XMSG>,
}

impl<XMSG> Default for Card<XMSG> {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            children: vec![],
            frame: Frame::default(),
        }
    }
}

impl<XMSG> Card<XMSG> {
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme.clone();
        self.frame.set_theme(theme);
    }
}

impl<XMSG> Container<Msg<XMSG>, XMSG> for Card<XMSG>
where
    XMSG: 'static,
{
    fn update(&mut self, msg: Msg<XMSG>) -> Effects<Msg<XMSG>, XMSG> {
        Effects::none()
    }

    fn view(&self, content: impl IntoIterator<Item = Node<XMSG>>) -> Node<Msg<XMSG>> {
        node! {
             <div class="card">
             {
                 self.frame.view([
                        node! {
                             <div class="card-body">
                                 <a href="#" class="product">
                                     <div class="img" style="background-image: url(assets/img/pos/product-1.jpg)"></div>
                                     <div class="info">
                                         <div class="title">"Grill Chicken Chop®"</div>
                                         <div class="desc">chicken, egg, mushroom, salad</div>
                                         <div class="price">$10.99</div>
                                     </div>
                                 </a>
                             </div>
                        }
                ]).map_msg(|fmsg|Msg::FrameMsg(fmsg))
             }
             </div>
        }
    }

    fn append_child(&mut self, child: Node<XMSG>) {
        self.children.push(child)
    }

    fn style(&self) -> Vec<String> {
        self.frame.style()
    }
}

#[custom_element("sfui-card")]
impl WebComponent<Msg<()>> for Card<()> {
    fn observed_attributes() -> Vec<&'static str> {
        vec!["theme-primary", "theme-background"]
    }

    fn attribute_changed(
        program: Program<Self, Msg<()>>,
        attr_name: &str,
        _old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let mut app = program.app_mut();
        match attr_name {
            "theme-primary" => {
                if let Some(primary) = new_value {
                    let background = &app.theme.background_color;
                    let theme =
                        Theme::from_str(&primary, background).expect("must be a valid theme");
                    app.set_theme(theme);
                }
            }
            "theme-background" => {
                if let Some(background) = new_value {
                    let primary = &app.theme.primary_color;
                    let theme =
                        Theme::from_str(primary, &background).expect("must be a valid theme");
                    app.set_theme(theme);
                }
            }
            _ => (),
        }
    }

    fn connected_callback(&mut self) {}
    fn disconnected_callback(&mut self) {}
    fn adopted_callback(&mut self) {}
}
