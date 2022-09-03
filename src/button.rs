//use css_colors::Color;
use crate::Theme;
use css_colors::Color;
use sauron::jss_ns_pretty;
use sauron::wasm_bindgen::JsCast;
use sauron::{
    dom::Callback,
    html::attributes,
    html::{attributes::class, div, events::on_click, text},
    prelude::*,
    Node,
};
use std::collections::BTreeMap;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlAudioElement;
use web_sys::MouseEvent;

const COMPONENT_NAME: &str = "sfui-button";
const DEFAULT_CHIPPED_BUTTON_WIDTH: usize = 200;
const DEFAULT_CHIPPED_BUTTON_HEIGHT: usize = 40;

#[derive(Clone, Debug)]
pub enum Msg {
    Click(MouseEvent),
    HoverIn,
    HoverOut,
    HighlightEnd,
    Mounted(MountEvent),
    ClickAudioMounted(web_sys::Node),
}

#[derive(Debug)]
pub struct Button<PMSG> {
    click_audio_src: String,
    click_audio: Option<HtmlAudioElement>,
    feature: Feature,
    label: String,
    click: bool,
    hover: bool,
    click_listeners: Vec<Callback<MouseEvent, PMSG>>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    component_id: Option<String>,
    theme: Theme,
}

#[derive(Debug)]
enum Pallete {
    Error,
    Success,
    Info,
    Warning,
}

impl Pallete {
    fn class_name(&self) -> &'static str {
        match self {
            Pallete::Error => "error",
            Pallete::Success => "success",
            Pallete::Info => "info",
            Pallete::Warning => "warning",
        }
    }
}

#[derive(Debug)]
pub struct Feature {
    pub hidden: bool,
    /// enable sound
    pub sound: bool,
    /// enable click effect, which changes the background color
    /// of the button with the highlight color
    pub click_highlights: bool,
    /// the button is slanted 45 degree to the right
    pub skewed: bool,
    /// has corners
    pub has_corners: bool,
    /// the button has borders
    pub has_borders: bool,
    /// enable/disable hover effect
    pub has_hover: bool,
    /// expand corners when hovered
    pub expand_corners: bool,
    /// the button is disabled
    pub disabled: bool,
    /// the bottom right of the button is chipped
    pub chipped: bool,
    /// the pallete color of the button
    pallete: Option<Pallete>,
}

impl<PMSG> Default for Button<PMSG>
where
    PMSG: 'static,
{
    fn default() -> Self {
        Button::with_label_and_theme("Button", Theme::default()).with_options(Feature::chipped())
    }
}

impl<PMSG> Button<PMSG>
where
    PMSG: 'static,
{
    pub fn with_label_and_theme(label: &str, theme: Theme) -> Self {
        Button {
            feature: Feature::default(),
            click_audio_src: "sounds/click.mp3".to_string(),
            click_audio: None,
            click: false,
            hover: false,
            label: label.to_string(),
            click_listeners: vec![],
            width: None,
            height: None,
            component_id: None,
            theme,
        }
    }

    fn view_actual_button(&self, width: Option<usize>, height: Option<usize>) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        button(
            [
                class_ns("button"),
                if let Some(ref pallete) = self.feature.pallete {
                    class_ns(pallete.class_name())
                } else {
                    empty_attr()
                },
                disabled(self.feature.disabled),
                if let Some(width) = width {
                    style! {width: px(width)}
                } else {
                    empty_attr()
                },
                if let Some(height) = height {
                    style! { height: px(height) }
                } else {
                    empty_attr()
                },
            ],
            [text(&self.label)],
        )
    }

    fn view_borders(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        node_list([
            view_if(
                self.feature.has_borders,
                div([class_ns("border border-left")], []),
            ),
            view_if(
                self.feature.has_borders,
                div([class_ns("border border-right")], []),
            ),
            view_if(
                self.feature.has_borders,
                div([class_ns("border border-top")], []),
            ),
            view_if(
                self.feature.has_borders,
                div([class_ns("border border-bottom")], []),
            ),
        ])
    }

    fn view_corners(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        node_list([
            view_if(
                self.feature.has_corners,
                div([class_ns("corner corner__top-left")], []),
            ),
            view_if(
                self.feature.has_corners,
                div([class_ns("corner corner__bottom-left")], []),
            ),
            view_if(
                self.feature.has_corners,
                div([class_ns("corner corner__top-right")], []),
            ),
            view_if(
                self.feature.has_corners,
                div([class_ns("corner corner__bottom-right")], []),
            ),
        ])
    }

    fn view_chipped_button(&self) -> Node<Msg> {
        let width = DEFAULT_CHIPPED_BUTTON_WIDTH;
        let height = DEFAULT_CHIPPED_BUTTON_HEIGHT;
        let (chip_width, chip_height) = (20, 20);
        let (gap_x, gap_y) = if self.hover { (8, 8) } else { (4, 4) };
        let top_left = (0, 0);
        let top_right = (width, 0);
        let bottom_left = (0, height);
        let chip1 = (width - chip_width, height);
        let chip2 = (width, height - chip_height);

        let poly_points = [bottom_left, chip1, chip2, top_right, top_left];

        let bottom_right = (width, height);
        let tri_edge1 = (width - chip_width + gap_x, height);
        let tri_edge2 = (width, height - chip_height + gap_y);
        let triangle = [tri_edge1, tri_edge2, bottom_right];

        let poly_points_str = poly_points
            .iter()
            .map(|p| format!("{},{}", p.0, p.1))
            .collect::<Vec<_>>()
            .join(" ");

        let triangle_points = triangle
            .iter()
            .map(|p| format!("{},{}", p.0, p.1))
            .collect::<Vec<_>>()
            .join(" ");

        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(
            [class_ns("chipped_wrapper")],
            [
                svg(
                    [
                        xmlns("http://www.w3.org/2000/svg"),
                        preserveAspectRatio("none"),
                        class_ns("chipped_svg"),
                        viewBox([0, 0, width, height]),
                    ],
                    [
                        polygon(
                            [
                                class_ns("chipped_polygon"),
                                points(poly_points_str),
                                on_transitionend(|_| Msg::HighlightEnd),
                            ],
                            [],
                        ),
                        polygon([class_ns("triangle"), points(triangle_points)], []),
                    ],
                ),
                button(
                    [
                        class_ns("chipped_button"),
                        disabled(self.feature.disabled),
                        style! {width: px(width)},
                        style! {height: px(height)},
                    ],
                    [text(&self.label)],
                ),
            ],
        )
    }
}

#[custom_element("sfui-button")]
impl<PMSG> Component<Msg, PMSG> for Button<PMSG>
where
    PMSG: 'static,
{
    fn get_component_id(&self) -> Option<&String> {
        self.component_id.as_ref()
    }

    /// what attributes this component is interested in
    fn observed_attributes() -> Vec<&'static str> {
        vec!["label", "theme-primary", "theme-background", "look"]
    }

    /// called when any of the attributes in observed_attributes is changed
    fn attributes_changed(&mut self, attributes_values: BTreeMap<String, String>) {
        log::info!("got some attributes changed: {:?}", attributes_values);
        for (attribute, value) in attributes_values {
            match attribute.as_ref() {
                "label" => self.label = value,
                "theme-primary" => {
                    let primary = &value;
                    let background = &self.theme.background_color;
                    self.theme =
                        Theme::from_str(primary, background).expect("must be a valid theme");
                }
                "theme-background" => {
                    let background = &value;
                    let primary = &self.theme.primary_color;
                    self.theme =
                        Theme::from_str(primary, background).expect("must be a valid theme");
                }
                "look" => match value.as_ref() {
                    "regular" => self.feature = Feature::regular(),
                    "skewed" => self.feature = Feature::skewed(),
                    "muted" => self.feature = Feature::muted(),
                    "chipped" => self.feature = Feature::chipped(),
                    "simple" => self.feature = Feature::simple(),
                    "disabled" => self.feature = Feature::disabled(),
                    _ => (),
                },
                _ => log::info!("some other attribute: {}", attribute),
            }
        }
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, PMSG> {
        match msg {
            Msg::Click(mouse_event) => {
                self.click = true;
                if self.feature.sound {
                    if let Some(audio) = &self.click_audio {
                        let promise = audio.play().expect("must play");
                        sauron::spawn_local(async move {
                            JsFuture::from(promise).await.expect("must not error");
                            log::info!("done playing..")
                        });
                    }
                }
                let pmsg_list = self
                    .click_listeners
                    .iter()
                    .map(|listener| listener.emit(mouse_event.clone()));
                Effects::with_external(pmsg_list)
            }
            Msg::HoverIn => {
                self.hover = true;
                Effects::none()
            }
            Msg::HoverOut => {
                self.hover = false;
                Effects::none()
            }
            Msg::HighlightEnd => {
                self.click = false;
                Effects::none()
            }
            Msg::Mounted(me) => {
                log::debug!("mounted...");
                log::info!("mounted: {:?}", me);
                let target_node = me.target_node;
                let target_elm: &web_sys::Element = target_node.dyn_ref().expect("must cast");
                if let Some(vdom_id) = target_elm.get_attribute("data-vdom-id") {
                    log::trace!("mounted: {}", vdom_id);
                    self.component_id = Some(vdom_id);
                }
                Effects::none()
            }
            Msg::ClickAudioMounted(node) => {
                let audio: HtmlAudioElement = node.unchecked_into();
                self.click_audio = Some(audio);
                Effects::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };

        div(
            [
                class(COMPONENT_NAME),
                classes_ns_flag([
                    ("clicked", self.click),
                    ("click_highlights", self.feature.click_highlights),
                    ("expand_corners", self.feature.expand_corners),
                    ("has_hover", self.feature.has_hover),
                    ("hovered", self.hover),
                    ("skewed", self.feature.skewed),
                    ("chipped", self.feature.chipped),
                    // setting this will also disable the div, therefore will not activate the
                    // events on it
                    ("disabled", self.feature.disabled),
                    ("hidden", self.feature.hidden),
                ]),
                if let Some(ref pallete) = self.feature.pallete {
                    class_ns(pallete.class_name())
                } else {
                    empty_attr()
                },
                // normally click should be attached to the actual button element
                on_click(Msg::Click),
                // the mouseover events are attached here since the hover element z-index is
                // higher than the actual button, which will cause a janky animation
                // when the mouse is triggering alt hover in and out, since covered by the hover
                // layer effect
                on_mouseover(|_| Msg::HoverIn),
                on_mouseout(|_| Msg::HoverOut),
                on_mount(|e| Msg::Mounted(e)),
            ],
            [
                audio(
                    [
                        on_mount(|me| Msg::ClickAudioMounted(me.target_node)),
                        src(&self.click_audio_src),
                    ],
                    [],
                ),
                // hover
                view_if(
                    self.feature.has_hover,
                    div([class_ns("hover hover-bottom")], []),
                ),
                // borders
                self.view_borders(),
                // corners
                self.view_corners(),
                if self.feature.chipped {
                    self.view_chipped_button()
                } else {
                    div(
                        [],
                        [
                            div(
                                [
                                    class_ns("highlight"),
                                    on_transitionend(|_| Msg::HighlightEnd),
                                ],
                                [],
                            ),
                            div(
                                [class_ns("button_wrap")],
                                [self.view_actual_button(self.width, self.height)],
                            ),
                        ],
                    )
                },
            ],
        )
    }

    fn style(&self) -> String {
        Self::main_style(&self.theme)
    }
}

impl<PMSG> Button<PMSG>
where
    PMSG: 'static,
{
    pub fn with_options(mut self, feature: Feature) -> Self {
        self.feature = feature;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    pub fn error(mut self) -> Self {
        self.feature.pallete = Some(Pallete::Error);
        self
    }

    pub fn success(mut self) -> Self {
        self.feature.pallete = Some(Pallete::Success);
        self
    }

    pub fn info(mut self) -> Self {
        self.feature.pallete = Some(Pallete::Info);
        self
    }

    pub fn warning(mut self) -> Self {
        self.feature.pallete = Some(Pallete::Warning);
        self
    }

    pub fn add_click_listener<F>(mut self, f: F) -> Self
    where
        F: Fn(MouseEvent) -> PMSG + 'static,
    {
        let cb = Callback::from(f);
        self.click_listeners.push(cb);
        self
    }

    fn border_style(theme: &crate::Theme) -> String {
        let border_width = 1; // the width of the border for each side of the button
        let base = &theme.controls;
        let transition_time_ms = 250; //transition time for most effects on the button

        jss_ns_pretty! {COMPONENT_NAME,
            // BORDERS these are styled divs wrapping the buttons
            ".border": {
                border_color: base.border_color.clone(),
                box_shadow: format!("{} {}",px([0,0,4]), base.border_shadow.clone()),
                z_index: 1,
                opacity: 1,
                position: "absolute",
                transition: format!("all {}ms ease-in",transition_time_ms),
                border_style: "solid",
            },

            ".error .border": {
                border_color: theme.pallete.error.to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.pallete.error.to_css()),
            },

            ".success .border": {
                border_color: theme.pallete.success.to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.pallete.success.to_css()),
            },

            ".info .border": {
                border_color: theme.pallete.info.to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.pallete.info.to_css()),
            },

            ".warning .border": {
                border_color: theme.pallete.warning.to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.pallete.warning.to_css()),
            },


            ".border-left": {
                top: percent(50),
                left: 0,
                height: percent(100),
                transform: format!("translate({}, {})", 0, percent(-50)),
                border_width: px([0, 0, 0, border_width]),
            },

            ".border-right": {
                top: percent(50),
                right: 0,
                height: percent(100),
                transform: format!("translate({}, {})", 0, percent(-50)),
                border_width: px([0, 0, 0, border_width]),
            },

            ".border-top": {
                top: 0,
                left: percent(50),
                width: percent(100),
                transform: format!("translate({}, {})", percent(-50), 0),
                border_width: px([border_width, 0, 0, 0]),
            },

            ".border-bottom": {
                left: percent(50),
                width: percent(100),
                bottom: 0,
                transform: format!("translate({}, {})", percent(-50), 0),
                border_width: px([border_width, 0, 0, 0]),
            },
        }
    }

    fn corner_style(theme: &Theme) -> String {
        let base = &theme.controls;
        let transition_time_ms = 250; //transition time for most effects on the button
        let corner_width = 2; // width of the corner clip of this button
        let corner_length = 8; // lengths of the corner clip of this button
        let corner_expand_distance = 6; // distance that clips at the corner expands when the button is hovered

        jss_ns_pretty! {COMPONENT_NAME,
            // CORNERS - the fancy divs which clips the button
            ".corner": {
                width: px(corner_length),
                height: px(corner_length),
                border_color: base.corner_color.clone(),
                box_shadow: format!("{} {}",px([0, 0, 4]), base.corner_shadow.clone()),
                z_index: 2,
                opacity: 1,
                position: "absolute",
                transition: format!("all {}ms ease-in",transition_time_ms),
                border_style: "solid",
            },

            ".error .corner": {
                border_color: theme.pallete.error.to_css(),
            },

            ".success .corner": {
                border_color: theme.pallete.success.to_css(),
            },

            ".info .corner": {
                border_color: theme.pallete.info.to_css(),
            },

            ".warning .corner": {
                border_color: theme.pallete.warning.to_css(),
            },


            ".corner__top-left": {
                left: px(-corner_width),
                top: px(-corner_width),
                border_width: px([corner_width, 0, 0, corner_width]),
            },

            ".corner__bottom-left": {
                left: px(-corner_width),
                bottom: px(-corner_width),
                border_width: px([0, 0, corner_width, corner_width]),
            },

            ".corner__top-right": {
                right: px(-corner_width),
                top: px(-corner_width),
                border_width: px([corner_width, corner_width, 0, 0]),
            },

            ".corner__bottom-right": {
                right: px(-corner_width),
                bottom: px(-corner_width),
                border_width: px([0, corner_width, corner_width, 0]),
            },

            // if expand_corners is enabled
            // the fui_button corners will EXPAND when hovered.
            //
            // CSS Notes:
            // - `.class1.class2 child` means if both class1 and class2 is specified in the
            // parent, the properties will be applied to this child element
            //
            //  - `.class1,.class2 child` means either if either class1 or class2 is specified in the
            // parent, the properties will be applied to this child element
            //
            ".expand_corners.hovered .corner__top-left": {
                left: px(-corner_expand_distance),
                top: px(-corner_expand_distance),
            },

            ".expand_corners.hovered .corner__bottom-left": {
                left: px(-corner_expand_distance),
                bottom: px(-corner_expand_distance),
            },

            ".expand_corners.hovered .corner__top-right": {
                right: px(-corner_expand_distance),
                top: px(-corner_expand_distance),
            },

            ".expand_corners.hovered .corner__bottom-right": {
                right: px(-corner_expand_distance),
                bottom: px(-corner_expand_distance),
            },
        }
    }

    pub fn main_style(theme: &Theme) -> String {
        let base = &theme.controls;
        let transition_time_ms = 250; //transition time for most effects on the button
        let hover_transition_time = 100; // the transition of the lower highligh of the button when hovering
        let highlight_transition = 50; // the transition time for the highlight color of the button when clicked

        let main = jss_ns_pretty! {COMPONENT_NAME,

            // the ROOT component style
            ".": {
                display: "inline-block",
                padding: px(1),
                position: "relative",
                margin: px([10, 10]),
            },

            ".hidden" : {
                visibility: "hidden",
            },

            // HOVER at the lower  part of the button
            ".hover": {
                border_color: base.hover_color.clone(),
                box_shadow: format!("{} {}", px([0,-2, 4]), base.hover_shadow.clone()),
                z_index: 4,
                opacity: 1,
                position: "absolute",
                transition: format!("width {}ms ease-in", hover_transition_time),
                border_style: "solid",
            },

            ".has_hover.hovered .hover": {
                width: percent(96),
            },

            ".has_hover.hovered.chipped .hover": {
                width: percent(80),
                transform: format!("skewX({}deg) translate({}, {})", -45, percent(-57), 0),
                transform_origin: "bottom left",
            },

            ".hover-bottom": {
                width: 0,
                left: percent(50),
                bottom: px(2),
                transform: format!("translate({}, {})",percent(-50), 0),
                border_width: px([4, 0, 0, 0]),
            },

            ".error .hover": {
                border_color: theme.pallete.error.to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.pallete.error.to_css()),
            },

            ".success .hover": {
                border_color: theme.pallete.success.to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.pallete.success.to_css()),
            },

            ".info .hover": {
                border_color: theme.pallete.info.to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.pallete.info.to_css()),
            },

            ".warning .hover": {
                border_color: theme.pallete.warning.to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.pallete.warning.to_css()),
            },

            ".button_wrap": {
                background_color: base.content_background_color.clone(),
                z_index: 3,
                display: "block",
                position: "relative",
                overflow: "hidden",
                transition: format!("background-color {}ms ease-in", transition_time_ms),
            },

            // The actual button
            ".button": {
                color: base.button_text_color.clone(),
                cursor: "pointer",
                margin: 0,
                border: "none",
                z_index: 2,
                display: "inline-block",
                padding: px([10, 20]),
                outline: "none",
                position: "relative",
                font_size: px(15.75),
                background_color: base.content_background_color.clone(),
                transition: format!("all {}ms ease-out", transition_time_ms),
                line_height: 1,
                user_select: "none",
                vertical_align: "middle",
            },

            ".error .button": {
                border_color: theme.pallete.error.to_css(),
            },

            ".success .button": {
                border_color: theme.pallete.success.to_css(),
            },

            ".info .button": {
                border_color: theme.pallete.info.to_css(),
            },

            ".warning .button": {
                border_color: theme.pallete.warning.to_css(),
            },

            ".chipped_wrapper": {
                position: "relative",
                width: px(DEFAULT_CHIPPED_BUTTON_WIDTH),
                height: px(DEFAULT_CHIPPED_BUTTON_HEIGHT),
            },

            // the svg of the chipped button
            ".chipped_svg": {
                width: px(DEFAULT_CHIPPED_BUTTON_WIDTH),
                height: px(DEFAULT_CHIPPED_BUTTON_HEIGHT),
                position: "absolute",
            },

            ".chipped_button": {
                color: base.button_text_color.clone(),
                position: "absolute",
                background_color: "transparent",
                border: 0,

                color: base.button_text_color.clone(),
                cursor: "pointer",
                margin: 0,
                border: "none",
                z_index: 2,
                display: "inline-block",
                padding: px([10, 20]),
                font_size: px(15.75),
                line_height: 1,
                user_select: "none",
                vertical_align: "middle",
            },

            ".chipped_polygon": {
                stroke_width: px(2),
                stroke: base.border_color.clone(),
                fill: base.content_background_color.clone(),
                vector_effect: "non-scaling-stroke",
                transition: format!("all {}ms ease-out", highlight_transition),
            },

            ".triangle": {
                stroke_width: px(2),
                fill: base.border_color.clone(),
                stroke: base.border_color.clone(),
            },

            ".error .chipped_polygon": {
                stroke: theme.pallete.error.to_css(),
            },

            ".success .chipped_polygon": {
                stroke: theme.pallete.success.to_css(),
            },

            ".info .chipped_polygon": {
                stroke: theme.pallete.info.to_css(),
            },

            ".warning .chipped_polygon": {
                stroke: theme.pallete.warning.to_css(),
            },


            ".error .triangle": {
                fill: theme.pallete.error.to_css(),
                stroke: theme.pallete.error.to_css(),
            },

            ".success .triangle": {
                fill: theme.pallete.success.to_css(),
                stroke: theme.pallete.success.to_css(),
            },

            ".info .triangle": {
                fill: theme.pallete.info.to_css(),
                stroke: theme.pallete.info.to_css(),
            },

            ".warning .triangle": {
                fill: theme.pallete.warning.to_css(),
                stroke: theme.pallete.warning.to_css(),
            },


            // highlight when clicked and fades out shortly
            ".click_highlights .highlight": {
                  z_index: 1,
                  position: "absolute",
                  left: 0,
                  right: 0,
                  top: 0,
                  bottom: 0,
                  background_color: base.highlight_color.clone(),
                  opacity: 0,
                  transition: format!("all {}ms ease-out", highlight_transition),
            },

            ".click_highlights.clicked .highlight": {
                opacity: 1,
            },

            ".click_highlights.clicked.error .highlight": {
                background_color: theme.pallete.error.to_css(),
            },

            ".click_highlights.clicked.success .highlight": {
                background_color: theme.pallete.success.to_css(),
            },

            ".click_highlights.clicked.info .highlight": {
                background_color: theme.pallete.info.to_css(),
            },

            ".click_highlights.clicked.warning .highlight": {
                background_color: theme.pallete.warning.to_css(),
            },

            ".click_highlights.clicked .chipped_polygon": {
                fill: base.highlight_color.clone(),
            },

            ".click_highlights.clicked.error .chipped_polygon": {
                fill: theme.pallete.error.to_css(),
            },

            ".click_highlights.clicked.success .chipped_polygon": {
                fill: theme.pallete.success.to_css(),
            },

            ".click_highlights.clicked.info .chipped_polygon": {
                fill: theme.pallete.info.to_css(),
            },

            ".click_highlights.clicked.warning .chipped_polygon": {
                fill: theme.pallete.warning.to_css(),
            },

            ".skewed": {
                transform: format!("skewX({}deg)", -45),
                transform_origin: "bottom left",
                margin_right: px(40),
            },

            ".skewed .button, .skewed .chipped_button": {
                transform: format!("skewX({}deg)", 45),
            },

        };

        [main, Self::border_style(theme), Self::corner_style(theme)].join("\n")
    }
}

impl Default for Feature {
    fn default() -> Self {
        Self {
            sound: true,
            click_highlights: true,
            skewed: false,
            has_corners: true,
            has_borders: true,
            expand_corners: true,
            has_hover: true,
            disabled: false,
            hidden: false,
            chipped: false,
            pallete: None,
        }
    }
}

impl Feature {
    /// has chipped at the bottom right
    pub fn chipped() -> Self {
        Feature {
            has_borders: false,
            chipped: true,
            ..Default::default()
        }
    }

    /// regular futuristic button
    /// don't expand corners
    #[allow(unused)]
    pub fn regular() -> Self {
        Feature::default()
    }

    pub fn skewed() -> Self {
        Self {
            skewed: true,
            ..Default::default()
        }
    }

    /// just like regular but muted
    /// sound off
    #[allow(unused)]
    pub fn muted() -> Self {
        Self {
            sound: false,
            ..Default::default()
        }
    }

    /// no corners, no hover
    #[allow(unused)]
    pub fn simple() -> Self {
        Self {
            has_corners: false,
            expand_corners: false,
            has_hover: false,
            ..Default::default()
        }
    }

    ///does not interact
    #[allow(unused)]
    pub fn disabled() -> Self {
        Feature {
            sound: false,
            click_highlights: false,
            skewed: false,
            has_corners: false,
            has_borders: true,
            expand_corners: false,
            has_hover: false,
            disabled: true,
            hidden: false,
            chipped: false,
            pallete: None,
        }
    }
}
