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
    theme: Theme,
    limit: usize,
}

impl<XMSG> Dice<XMSG>
where
    XMSG: 'static,
{
    pub fn new() -> Self {
        let width = 500.0;
        let height = 500.0;
        let slice_size = 40.0;
        let gap = 1.0;

        let properties = Properties {
            width,
            height,
            slice_size,
            gap,
        };

        Dice {
            audio_src: "sounds/typing.mp3".to_string(),
            audio: None,
            click_listeners: vec![],
            properties,
            is_animating: false,
            theme: Theme::default(),
            limit: 0,
        }
    }

    fn slice_view(
        &self,
        content: impl IntoIterator<Item = Node<Msg<XMSG>>> + Clone,
    ) -> Node<Msg<XMSG>> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let mut cells = vec![];
        let (slice_x, slice_y) = self.properties.slices();
        let max = slice_x * slice_y;
        //let limit = if let Some(limit) = limit { limit } else { max };
        let mut index = 0;
        let slice_size = self.properties.slice_size;
        for y in 0..slice_y {
            let top = (self.properties.slice_size + self.properties.gap) * y as f32;
            for x in 0..slice_x {
                if index < self.limit {
                    let left = (self.properties.slice_size + self.properties.gap) * x as f32;
                    let cell = div(
                        [
                            class_ns("slice"),
                            style! {
                                position: "absolute",
                                left: px(left),
                                top: px(top),
                                overflow: "hidden",
                                width: px(slice_size),
                                height: px(slice_size),
                            },
                        ],
                        [div(
                            [
                                class_ns("slice_offset"),
                                style! {
                                    position: "absolute",
                                    left: px(-left),
                                    top: px(-top),
                                },
                            ],
                            content.clone(),
                        )],
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
                Effects::none()
            }
            Msg::NextAnimation(start, duration) => {
                log::info!("in next animation... ");

                let timestamp = sauron::now();

                // the time that has elapsed since the start
                let anim_progress = (timestamp - start).max(0.0);

                let content_len = self.content_len();
                // how many of the slice that are already rendered
                self.limit = (anim_progress * content_len as f64 / duration).round() as usize;

                let continue_animation = self.limit <= content_len - 1;

                if continue_animation {
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

        let content_effect = if self.is_animating {
            Some(self.slice_view(content_node.clone()))
        } else {
            None
        };

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
                    if let Some(content_effect) = content_effect {
                        vec![content_effect]
                    } else {
                        vec![]
                    },
                ),
                div(
                    [if self.is_animating {
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
            ".animating .img": {
                opacity: 0,
            },
            ".slice": {
                  width: px(self.slice_size),
                  height: px(self.slice_size),
                  position: "absolute",
            }
        }
    }
}

impl<XMSG> Dice<XMSG> {
    fn content_len(&self) -> usize {
        self.properties.content_len()
    }
}
