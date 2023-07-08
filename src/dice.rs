use crate::Theme;
use async_trait::async_trait;
use sauron::wasm_bindgen::JsCast;
use sauron::{
    dom::{spawn_local, Callback},
    html::{attributes::*, events::*, *},
    *,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlAudioElement;

const COMPONENT_NAME: &str = "sfui-dice";

#[derive(Clone, Debug)]
pub enum Msg<XMSG> {
    AnimateIn,
    StopAnimation,
    NextAnimation(f64, f64),
    External(XMSG),
    ContainerMounted(MountEvent),
    AudioMounted(web_sys::Node),
}

pub struct Properties {
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
    width: Option<f32>,
    height: Option<f32>,
}

impl<XMSG> Dice<XMSG>
where
    XMSG: 'static,
{
    pub fn new() -> Self {
        let slice_size = 40.0;
        let gap = 1.0;

        let properties = Properties { slice_size, gap };

        Dice {
            audio_src: "sounds/typing.mp3".to_string(),
            audio: None,
            click_listeners: vec![],
            properties,
            is_animating: false,
            theme: Theme::default(),
            limit: 0,
            width: None,
            height: None,
        }
    }

    fn slice_view(
        &self,
        content: impl IntoIterator<Item = Node<Msg<XMSG>>> + Clone,
    ) -> Node<Msg<XMSG>> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let mut cells = vec![];
        let (slice_x, slice_y) = self.slices();
        let max = slice_x * slice_y;
        log::debug!(
            "computed_width: {}x{}",
            self.computed_width(),
            self.computed_height()
        );
        log::info!("slice_x: {}, slice_y: {}", slice_x, slice_y);
        log::info!("max slices: {}", max);
        let mut index = 0;
        let slice_size = self.properties.slice_size;
        log::info!("limit: {}", self.limit);
        for y in 0..slice_y {
            let top = (self.properties.slice_size + self.properties.gap) * y as f32;
            for x in 0..slice_x {
                let border = if index < self.limit {
                    "1px solid red"
                } else {
                    "1px solid green"
                };
                let visibility = if index < self.limit {
                    "visible"
                } else {
                    "hidden"
                };
                {
                    let left = (self.properties.slice_size + self.properties.gap) * x as f32;
                    let cell = div(
                        [
                            class_ns("slice"),
                            style! {
                                position: "absolute",
                                border: border,
                                visibility: visibility,
                                left: px(left),
                                top: px(top),
                                width: px(slice_size),
                                height: px(slice_size),
                            },
                        ],
                        [div(
                            [
                                class_ns("slice_offset"),
                                style! {
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
    fn init(&mut self) -> Vec<Task<Msg<XMSG>>> {
        vec![]
    }
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

                if let Some(audio) = &self.audio {
                    let promise = audio.play().expect("must play");
                    spawn_local(async move {
                        JsFuture::from(promise).await.expect("must not error");
                    });
                }

                Effects::with_local([Msg::NextAnimation(start, duration)])
            }
            Msg::StopAnimation => {
                self.is_animating = false;
                self.limit = 0;
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
                log::info!(
                    "next animation limit: {}, timestamp: {}",
                    self.limit,
                    timestamp
                );

                let continue_animation = self.limit <= content_len - 1;

                if continue_animation {
                    Effects::with_local([Msg::NextAnimation(start, duration)])
                } else {
                    Effects::with_local([Msg::StopAnimation])
                }
            }
            Msg::External(xmsg) => Effects::with_external(vec![xmsg]),
            Msg::ContainerMounted(me) => {
                let container_element: web_sys::Element = me.target_node.unchecked_into();
                //TODO: this is not accurately computing the container
                //dimension, maybe because of the container not included the overflow of the
                //content
                let rect = container_element.get_bounding_client_rect();
                self.width = Some(rect.width() as f32);
                self.height = Some(rect.height() as f32);
                Effects::none()
            }
            Msg::AudioMounted(node) => {
                let audio: HtmlAudioElement = node.unchecked_into();
                self.audio = Some(audio);
                Effects::none()
            }
        }
    }

    fn view(&self, content: impl IntoIterator<Item = Node<XMSG>>) -> Node<Msg<XMSG>> {
        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
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
            ],
            [
                audio(
                    [
                        on_mount(|me| Msg::AudioMounted(me.target_node)),
                        src(&self.audio_src),
                    ],
                    [],
                ),
                div(
                    [class_ns("effect")],
                    if let Some(content_effect) = content_effect {
                        vec![content_effect]
                    } else {
                        vec![]
                    },
                ),
                div(
                    [
                        class_ns("container"),
                        on_mount(|me| Msg::ContainerMounted(me)),
                        if self.is_animating {
                            style! { visibility: "hidden" }
                        } else {
                            empty_attr()
                        },
                    ],
                    content_node,
                ),
            ],
        )
    }

    fn style(&self) -> Vec<String> {
        self.properties.style(&self.theme)
    }

    fn append_child(&mut self, child: Node<XMSG>) {}
}

impl Properties {
    fn style(&self, theme: &crate::Theme) -> Vec<String> {
        vec![jss_ns! {COMPONENT_NAME,
            ".": {
                display: "inline-block",
                position: "relative",
            },
            ".effects_slices": {
                display: "block",
                position: "relative",
            },
            ".animating .img": {
                opacity: 0,
            },
            ".slice": {
                position: "absolute",
                overflow: "hidden",
            },
            ".slice_offset" : {
                position: "absolute",
            },

            ".container": {
                display: "inline-block",
                overflow: "hidden",
            }
        }]
    }
}

impl<XMSG> Dice<XMSG> {
    /// slices on x and slices on y
    fn slices(&self) -> (usize, usize) {
        let prop = &self.properties;
        (
            (self.computed_width() / (prop.slice_size + prop.gap)).round() as usize,
            (self.computed_height() / (prop.slice_size + prop.gap)).round() as usize,
        )
    }

    fn content_len(&self) -> usize {
        let (w, h) = self.slices();
        w * h
    }

    fn computed_width(&self) -> f32 {
        if let Some(width) = self.width {
            width
        } else {
            0.0
        }
    }

    fn computed_height(&self) -> f32 {
        if let Some(height) = self.height {
            height
        } else {
            0.0
        }
    }
}
