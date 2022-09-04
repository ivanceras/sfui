use crate::Status;
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

const COMPONENT_NAME: &str = "sfui-frame";
const DEFAULT_CHIPPED_BUTTON_WIDTH: usize = 100;
const DEFAULT_CHIPPED_BUTTON_HEIGHT: usize = 40;

#[derive(Clone, Debug)]
pub enum Msg<PMSG> {
    Click(MouseEvent),
    HoverIn,
    HoverOut,
    HighlightEnd,
    External(PMSG),
}

#[derive(Debug)]
pub struct Frame<PMSG> {
    feature: Feature,
    label: String,
    clicked: bool,
    hovered: bool,
    click_listeners: Vec<Callback<MouseEvent, PMSG>>,
    width: Option<usize>,
    height: Option<usize>,
    theme: Theme,
    /// the status of the button which changes the color pallet of the button
    status: Option<Status>,
    content: Option<Node<PMSG>>,
}

#[derive(Debug)]
pub struct Feature {
    /// enable click effect, which changes the background color
    /// of the button with the highlight color
    pub click_highlights: bool,
    /// has corners
    pub has_corners: bool,
    /// the frame has borders
    pub has_borders: bool,
    /// expand corners when hovered
    pub expand_corners: bool,
    pub has_corner_box_shadow: bool,
}

pub struct Dimension {
    /// width of the corner clip of this button
    pub corner_width: usize,
    /// lengths of the corner clip of this button
    pub corner_length: usize,
    /// distance that clips at the corner expands when the button is hovered
    pub corner_expand_distance: usize,
}

impl Default for Dimension {
    fn default() -> Self {
        Self {
            corner_width: 4,
            corner_length: 16,
            corner_expand_distance: 12,
        }
    }
}

impl Dimension {
    pub fn small() -> Self {
        Self {
            corner_width: 2,
            corner_length: 8,
            corner_expand_distance: 6,
        }
    }
}

impl<PMSG> Default for Frame<PMSG>
where
    PMSG: 'static,
{
    fn default() -> Self {
        Self {
            feature: Feature::default(),
            clicked: false,
            hovered: false,
            label: "Frame".to_string(),
            click_listeners: vec![],
            width: None,
            height: None,
            theme: Theme::default(),
            status: None,
            content: None,
        }
    }
}

impl<PMSG> Frame<PMSG>
where
    PMSG: 'static,
{
    pub fn with_label(label: &str) -> Self {
        Self {
            label: label.to_string(),
            ..Default::default()
        }
    }

    pub fn with_content(mut self, content: Node<PMSG>) -> Self {
        self.content = Some(content);
        self
    }

    fn computed_width(&self) -> usize {
        // use the supplied width if it is specified
        if let Some(width) = self.width {
            width
        } else {
            // otherwise calculate it
            let font_width = 10;
            let label_width = self.label.len() * font_width;
            std::cmp::max(DEFAULT_CHIPPED_BUTTON_WIDTH, label_width)
        }
    }

    fn computed_height(&self) -> usize {
        if let Some(height) = self.height {
            height
        } else {
            DEFAULT_CHIPPED_BUTTON_HEIGHT
        }
    }

    fn view_borders(&self) -> Node<Msg<PMSG>> {
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

    fn view_corners(&self) -> Node<Msg<PMSG>> {
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
}

//#[custom_element("sfui-frame")]
impl<PMSG> Container<Msg<PMSG>, PMSG> for Frame<PMSG>
where
    PMSG: 'static,
{
    fn update(&mut self, msg: Msg<PMSG>) -> Effects<Msg<PMSG>, PMSG> {
        match msg {
            Msg::Click(mouse_event) => {
                self.clicked = true;
                let pmsg_list = self
                    .click_listeners
                    .iter()
                    .map(|listener| listener.emit(mouse_event.clone()));
                Effects::with_external(pmsg_list)
            }
            Msg::HoverIn => {
                self.hovered = true;
                Effects::none()
            }
            Msg::HoverOut => {
                self.hovered = false;
                Effects::none()
            }
            Msg::HighlightEnd => {
                self.clicked = false;
                Effects::none()
            }
            Msg::External(xmsg) => Effects::with_external([xmsg]),
        }
    }

    fn view(&self, content: impl IntoIterator<Item = Node<PMSG>>) -> Node<Msg<PMSG>> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);

        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };

        div(
            [
                class(COMPONENT_NAME),
                classes_ns_flag([
                    ("clicked", self.clicked),
                    ("click_highlights", self.feature.click_highlights),
                    ("expand_corners", self.feature.expand_corners),
                    ("has_corner_box_shadow", self.feature.has_corner_box_shadow),
                    ("hovered", self.hovered),
                ]),
                if let Some(ref status) = self.status {
                    class_ns(status.class_name())
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
            ],
            [div(
                [],
                [
                    // borders
                    self.view_borders(),
                    // corners
                    self.view_corners(),
                    div(
                        [],
                        /*
                        if let Some(content) = &self.content {
                            vec![content.clone().map_msg(Msg::External)]
                        } else {
                            vec![]
                        },
                        */
                        content.into_iter().map(|node| node.map_msg(Msg::External)),
                    ),
                ],
            )],
        )
    }

    fn style(&self) -> String {
        let theme = &self.theme;
        let base = &theme.controls;
        let transition_time_ms = self.transition_time_ms(); //transition time for most effects on the button
        let hover_transition_time = 100; // the transition of the lower highligh of the button when hovering
        let highlight_transition = 50; // the transition time for the highlight color of the button when clicked

        let width = self.computed_width();
        let height = self.computed_height();

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

            // hover effect at the lower part of the button
            ".underline": {
                border_color: base.hover_color.clone(),
                box_shadow: format!("{} {}", px([0,-2, 4]), base.hover_shadow.clone()),
                z_index: 4,
                opacity: 1,
                position: "absolute",
                transition: format!("width {}ms ease-in", hover_transition_time),
                border_style: "solid",
            },

            ".has_underline.hovered .underline": {
                width: percent(96),
            },

            ".has_underline.hovered.chipped .underline": {
                width: percent(70),
                transform: format!("skewX({}deg) translate({}, {})", -45, percent(-57), 0),
                transform_origin: "bottom left",
            },

            ".underline-bottom": {
                width: 0,
                left: percent(50),
                bottom: px(2),
                transform: format!("translate({}, {})",percent(-50), 0),
                border_width: px([4, 0, 0, 0]),
            },

            ".error .underline": {
                border_color: theme.error().to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.error().to_css()),
            },

            ".success .underline": {
                border_color: theme.success().to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.success().to_css()),
            },

            ".info .underline": {
                border_color: theme.info().to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.info().to_css()),
            },

            ".warning .underline": {
                border_color: theme.warning().to_css(),
                box_shadow: format!("{} {}",px([0, -2, 4]), theme.warning().to_css()),
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
                white_space: "nowrap",
            },

            ".error .button": {
                border_color: theme.error().to_css(),
            },

            ".success .button": {
                border_color: theme.success().to_css(),
            },

            ".info .button": {
                border_color: theme.info().to_css(),
            },

            ".warning .button": {
                border_color: theme.warning().to_css(),
            },

            ".chipped_wrapper": {
                position: "relative",
                width: px(width),
                height: px(height),
            },

            // the svg of the chipped button
            ".chipped_svg": {
                width: px(width),
                height: px(height),
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
                white_space: "nowrap",
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
                stroke: theme.error().to_css(),
            },

            ".success .chipped_polygon": {
                stroke: theme.success().to_css(),
            },

            ".info .chipped_polygon": {
                stroke: theme.info().to_css(),
            },

            ".warning .chipped_polygon": {
                stroke: theme.warning().to_css(),
            },


            ".error .triangle": {
                fill: theme.error().to_css(),
                stroke: theme.error().to_css(),
            },

            ".success .triangle": {
                fill: theme.success().to_css(),
                stroke: theme.success().to_css(),
            },

            ".info .triangle": {
                fill: theme.info().to_css(),
                stroke: theme.info().to_css(),
            },

            ".warning .triangle": {
                fill: theme.warning().to_css(),
                stroke: theme.warning().to_css(),
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
                background_color: theme.error().to_css(),
            },

            ".click_highlights.clicked.success .highlight": {
                background_color: theme.success().to_css(),
            },

            ".click_highlights.clicked.info .highlight": {
                background_color: theme.info().to_css(),
            },

            ".click_highlights.clicked.warning .highlight": {
                background_color: theme.warning().to_css(),
            },

            ".click_highlights.clicked .chipped_polygon": {
                fill: base.highlight_color.clone(),
            },

            ".click_highlights.clicked.error .chipped_polygon": {
                fill: theme.error().to_css(),
            },

            ".click_highlights.clicked.success .chipped_polygon": {
                fill: theme.success().to_css(),
            },

            ".click_highlights.clicked.info .chipped_polygon": {
                fill: theme.info().to_css(),
            },

            ".click_highlights.clicked.warning .chipped_polygon": {
                fill: theme.warning().to_css(),
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

        [main, self.border_style(), self.corner_style()].join("\n")
    }
}

impl<PMSG> Frame<PMSG>
where
    PMSG: 'static,
{
    pub fn with_options(mut self, feature: Feature) -> Self {
        self.feature = feature;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
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

    ///transition time for most effects on the frame
    fn transition_time_ms(&self) -> usize {
        250
    }

    fn border_style(&self) -> String {
        let theme = &self.theme;
        let border_width = 1; // the width of the border for each side of the button
        let base = &theme.controls;
        let transition_time_ms = self.transition_time_ms(); //transition time for most effects on the button

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
                border_color: theme.error().to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.error().to_css()),
            },

            ".success .border": {
                border_color: theme.success().to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.success().to_css()),
            },

            ".info .border": {
                border_color: theme.info().to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.info().to_css()),
            },

            ".warning .border": {
                border_color: theme.warning().to_css(),
                box_shadow: format!("{} {}",px([0, 0, 4]), theme.warning().to_css()),
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

    fn corner_style(&self) -> String {
        let theme = &self.theme;
        let base = &theme.controls;
        let transition_time_ms = self.transition_time_ms(); //transition time for most effects on the button
        let corner_width = 4; // width of the corner clip of this button
        let corner_length = 16; // lengths of the corner clip of this button
        let corner_expand_distance = 12; // distance that clips at the corner expands when the button is hovered

        jss_ns_pretty! {COMPONENT_NAME,
            // CORNERS - the fancy divs which clips the button
            ".corner": {
                width: px(corner_length),
                height: px(corner_length),
                border_color: base.corner_color.clone(),
                z_index: 2,
                opacity: 1,
                position: "absolute",
                transition: format!("all {}ms ease-in",transition_time_ms),
                border_style: "solid",
            },

            ".has_corner_box_shadow .corner": {
                box_shadow: format!("{} {}",px([0, 0, 4]), base.corner_shadow.clone()),
            },

            ".error .corner": {
                border_color: theme.error().to_css(),
            },

            ".success .corner": {
                border_color: theme.success().to_css(),
            },

            ".info .corner": {
                border_color: theme.info().to_css(),
            },

            ".warning .corner": {
                border_color: theme.warning().to_css(),
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
}

impl Default for Feature {
    fn default() -> Self {
        Self {
            click_highlights: true,
            has_corners: true,
            has_borders: false,
            expand_corners: true,
            has_corner_box_shadow: false,
        }
    }
}

pub fn register() {
    log::info!("registering..");
}
