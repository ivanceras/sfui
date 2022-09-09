use crate::Theme;
use sauron::{
    html::{attributes, div},
    jss_ns,
    prelude::*,
    Node,
};
use web_sys::HtmlAudioElement;

const COMPONENT_NAME: &str = "sfui-dice";

#[derive(Clone, Debug)]
pub enum Msg<XMSG> {
    AnimateIn,
    StopAnimation,
    NextAnimation(f64, f64),
    External(XMSG),
}

pub struct Properties {
    url: String,
    width: f32,
    height: f32,
    //slice_size size should be square
    slice_size: f32,
    gap: f32,
}

pub struct Dice<XMSG> {
    audio_src: String,
    audio: Option<HtmlAudioElement>,
    click_listeners: Vec<Callback<MouseEvent, XMSG>>,
    properties: Properties,
    is_animating: bool,
    content_effect: Option<Node<Msg<XMSG>>>,
    theme: Theme,
}

impl<XMSG> Dice<XMSG> {
    pub fn new(url: impl ToString) -> Self {
        log::info!("url: {}", url.to_string());
        let width = 1000.0;
        let height = 600.0;
        let slice_size = 40.0;
        let gap = 1.0;

        let properties = Properties {
            width,
            height,
            slice_size,
            gap,
            url: url.to_string(),
        };

        Dice {
            audio_src: "sounds/typing.mp3".to_string(),
            audio: None,
            click_listeners: vec![],
            properties,
            is_animating: false,
            content_effect: None,
            theme: Theme::default(),
        }
    }

    fn slice_view(&self, limit: Option<usize>) -> Node<Msg<XMSG>> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let mut cells = vec![];
        let (slice_x, slice_y) = self.properties.slices();
        let max = slice_x * slice_y;
        let limit = if let Some(limit) = limit { limit } else { max };
        let mut index = 0;
        for y in 0..slice_y {
            let top = (self.properties.slice_size + self.properties.gap) * y as f32;
            for x in 0..slice_x {
                if index < limit {
                    let left = (self.properties.slice_size + self.properties.gap) * x as f32;
                    let cell = div(
                        [
                            class_ns("slice"),
                            style! {
                                left: px(left),
                                top: px(top),
                                background_position: format!("{} {}", px(-left), px(-top)),
                            },
                        ],
                        [],
                    );
                    cells.push(cell);
                }
                index += 1;
            }
        }
        div([class_ns("effects_slices")], cells)
    }
}

impl<XMSG> Container<Msg<XMSG>, XMSG> for Dice<XMSG>
where
    XMSG: 'static,
{
    fn update(&mut self, msg: Msg<XMSG>) -> Effects<Msg<XMSG>, XMSG> {
        match msg {
            Msg::AnimateIn => {
                log::info!("starting the animation");
                let interval = 1_000.0 / 60.0;
                let real_duration = interval * self.content_len() as f64;
                let timeout = 500.0;
                let duration = real_duration.min(timeout);
                let start = sauron::now();

                self.is_animating = true;

                Effects::with_local([Msg::NextAnimation(start, duration)])
            }
            Msg::StopAnimation => {
                self.is_animating = false;
                self.content_effect = None;
                Effects::none()
            }
            Msg::NextAnimation(start, duration) => {
                log::info!("in next animation... ");

                let timestamp = sauron::now();

                // the time that has elapsed since the start
                let anim_progress = (timestamp - start).max(0.0);

                let content_len = self.content_len();
                // how many of the slice that are already rendered
                let limit = (anim_progress * content_len as f64 / duration).round() as usize;

                let continue_animation = limit <= content_len - 1;

                if continue_animation {
                    self.content_effect = Some(self.slice_view(Some(limit)));
                    Effects::with_local([Msg::NextAnimation(start, duration)])
                } else {
                    Effects::with_local([Msg::StopAnimation])
                }
            }
            Msg::External(xmsg) => Effects::with_external(vec![xmsg]),
        }
    }

    fn view(&self, content: impl IntoIterator<Item = Node<XMSG>>) -> Node<Msg<XMSG>> {
        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };
        let content_node = content
            .into_iter()
            .map(|node| node.map_msg(Msg::External))
            .collect::<Vec<_>>();

        div(
            [
                class(COMPONENT_NAME),
                classes_ns_flag([("animating", self.is_animating)]),
                on_click(|_| Msg::AnimateIn),
                //on_mouseout(|_| Msg::AnimateIn),
            ],
            [
                div(
                    [class("effect")],
                    if let Some(content_effect) = &self.content_effect {
                        vec![content_effect.clone()]
                    } else {
                        vec![]
                    },
                ),
                div(
                    [if self.content_effect.is_some() {
                        style! { visibility: "hidden" }
                    } else {
                        empty_attr()
                    }],
                    content_node,
                ),
            ],
        )
    }

    fn style(&self) -> String {
        self.properties.style(&self.theme)
    }

    fn append_child(&mut self, child: Node<XMSG>) {}
}

impl Properties {
    /// slices on x and slices on y
    fn slices(&self) -> (usize, usize) {
        (
            (self.width / (self.slice_size + self.gap)).round() as usize,
            (self.height / (self.slice_size + self.gap)).round() as usize,
        )
    }

    fn content_len(&self) -> usize {
        let (w, h) = self.slices();
        w * h
    }

    fn style(&self, theme: &crate::Theme) -> String {
        jss_ns! {COMPONENT_NAME,
            ".": {
                display: "inline-block",
                width: px(self.width),
                height: px(self.height),
                position: "relative",
            },
            ".effects_slices": {
                display: "inline-block",
                width: px(self.width),
                height: px(self.height),
                position: "relative",
            },
            ".img": {
                width: px(self.width),
                height: px(self.height),
                position: "relative",
                opacity: 1,
                background_size: format!("{} {}", px(self.width), px(self.height)),
                background_image: format!("linear-gradient({} 0, {} 25%, {} 75%, {} 100%), url({})"
                        ,theme.background_color, theme.primary_color, theme.accent_color, theme.background_color, self.url),
                background_blend_mode: "color",
            },
            ".animating .img": {
                opacity: 0,
            },
            ".slice": {
                  width: px(self.slice_size),
                  height: px(self.slice_size),
                  position: "absolute",
                  background_size: format!("{} {}", px(self.width), px(self.height)),
                  background_image: format!("linear-gradient({} 0, {} 25%, {} 75%, {} 100%), url({})"
                      ,theme.background_color, theme.primary_color, theme.accent_color, theme.background_color, self.url),
                  background_repeat:"no-repeat no-repeat",
                  background_attachment: "local, local",
                  background_blend_mode: "color",
            }
        }
    }
}

impl<XMSG> Dice<XMSG> {
    fn content_len(&self) -> usize {
        self.properties.content_len()
    }
}
