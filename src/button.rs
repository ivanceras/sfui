use crate::frame::{self, Dimension, Frame};
use crate::Status;
use crate::Theme;
use async_trait::async_trait;
use css_colors::Color;
use sauron::jss_ns_pretty;
use sauron::wasm_bindgen::JsCast;
use sauron::{
    dom::{register_custom_element, spawn_local, Callback},
    html::attributes,
    html::{attributes::*, events::*, *},
    svg::attributes::{points, preserve_aspect_ratio, view_box, xmlns},
    *,
};
use std::collections::BTreeMap;
use std::str::FromStr;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlAudioElement;
use web_sys::MouseEvent;

const COMPONENT_NAME: &str = "sfui-button";
const DEFAULT_CHIPPED_BUTTON_WIDTH: i32 = 100;
const DEFAULT_CHIPPED_BUTTON_HEIGHT: i32 = 40;

#[derive(Clone, Debug)]
pub enum Msg {
    Click(MouseEvent),
    HoverIn,
    HoverOut,
    HighlightEnd,
    ClickAudioMounted(web_sys::Node),
    FrameMsg(Box<frame::Msg<Msg>>),
    ChippedButtonMounted(MountEvent),
}

#[derive(Debug)]
pub struct Button<XMSG> {
    click_audio_src: String,
    click_audio: Option<HtmlAudioElement>,
    feature: Feature,
    label: String,
    clicked: bool,
    hovered: bool,
    click_listeners: Vec<Callback<MouseEvent, XMSG>>,
    width: Option<i32>,
    height: Option<i32>,
    theme: Theme,
    /// the status of the button which changes the color pallet of the button
    status: Option<Status>,
    frame: Frame<Msg>,
    chipped_button: Option<web_sys::Element>,
    /// used by chipped_button
    button_width: Option<f32>,
    button_height: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
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
    pub has_underline: bool,
    /// expand corners when hovered
    pub expand_corners: bool,
    pub has_corner_box_shadow: bool,
    /// the button is disabled
    pub disabled: bool,
    /// the bottom right of the button is chipped
    pub chipped: bool,
}

impl FromStr for Feature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "regular" => Ok(Feature::regular()),
            "skewed" => Ok(Feature::skewed()),
            "muted" => Ok(Feature::muted()),
            "chipped" => Ok(Feature::chipped()),
            "simple" => Ok(Feature::simple()),
            "disabled" => Ok(Feature::disabled()),
            _ => Err(()),
        }
    }
}

impl Into<frame::Feature> for Feature {
    fn into(self) -> frame::Feature {
        frame::Feature {
            has_corners: self.has_corners,
            has_borders: self.has_borders,
            expand_corners: self.expand_corners,
            has_corner_box_shadow: self.has_corner_box_shadow,
        }
    }
}

impl<XMSG> Default for Button<XMSG>
where
    XMSG: 'static,
{
    fn default() -> Self {
        let mut frame = Frame::<Msg>::default();
        frame.set_dimension(Dimension::small());
        Self {
            feature: Feature::chipped(),
            click_audio_src: "sounds/click.mp3".to_string(),
            click_audio: None,
            clicked: false,
            hovered: false,
            label: "Button".to_string(),
            click_listeners: vec![],
            width: None,
            height: None,
            theme: Theme::default(),
            status: None,
            frame,
            chipped_button: None,
            button_width: None,
            button_height: None,
        }
    }
}

impl<XMSG> Button<XMSG>
where
    XMSG: 'static,
{
    pub fn with_label(label: &str) -> Self {
        Self {
            label: label.to_string(),
            ..Default::default()
        }
    }

    pub fn set_feature(&mut self, feature: Feature) {
        self.feature = feature;
        self.frame.set_feature(feature.into());
    }

    fn computed_width(&self) -> i32 {
        // use the supplied width if it is specified
        if let Some(width) = self.width {
            width
        } else {
            // otherwise calculate it
            let font_width = 10;
            let label_width = self.label.len() as i32 * font_width;
            std::cmp::max(DEFAULT_CHIPPED_BUTTON_WIDTH, label_width)
        }
    }

    fn computed_height(&self) -> i32 {
        if let Some(height) = self.height {
            height
        } else {
            DEFAULT_CHIPPED_BUTTON_HEIGHT
        }
    }

    fn button_width(&self) -> f32 {
        if let Some(button_width) = self.button_width {
            button_width
        } else {
            self.computed_width() as f32
        }
    }

    fn button_height(&self) -> f32 {
        if let Some(button_height) = self.button_height {
            button_height
        } else {
            self.computed_height() as f32
        }
    }

    fn view_button(&self) -> Node<Msg> {
        if self.feature.chipped {
            self.view_chipped_button()
        } else {
            self.view_plain_button()
        }
    }

    fn view_plain_button(&self) -> Node<Msg> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
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
                    [button(
                        [
                            class_ns("button"),
                            if let Some(ref status) = self.status {
                                class_ns(status.class_name())
                            } else {
                                empty_attr()
                            },
                            disabled(self.feature.disabled),
                            if let Some(width) = self.width {
                                style! {width: px(width)}
                            } else {
                                empty_attr()
                            },
                            if let Some(height) = self.height {
                                style! { height: px(height) }
                            } else {
                                empty_attr()
                            },
                        ],
                        [text(&self.label)],
                    )],
                ),
            ],
        )
    }

    fn view_chipped_button(&self) -> Node<Msg> {
        let width = self.computed_width();
        let height = self.computed_height();

        let (chip_width, chip_height) = (20, 20);
        let (gap_x, gap_y) = if self.hovered { (8, 8) } else { (4, 4) };
        let top_left = (0, 0);
        let top_right = (width, 0);
        let bottom_left = (0, height);
        let chip1 = (width - chip_width, height);
        let chip2 = (width, height - chip_height);

        let poly_points = [bottom_left, chip1, chip2, top_right, top_left];

        let bottom_right = (width, height);

        //     /
        //    *-
        let tri_edge1 = (width - chip_width + gap_x, height);

        //      *
        //     /|
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
                        preserve_aspect_ratio("none"),
                        class_ns("chipped_svg"),
                        view_box([0, 0, width, height]),
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
                        on_mount(Msg::ChippedButtonMounted),
                    ],
                    [text(&self.label)],
                ),
            ],
        )
    }
}

// Note: we are not using the custom element macro yet
// since, there are hiccups at the moment
impl<XMSG> Component<Msg, XMSG> for Button<XMSG>
where
    XMSG: 'static,
{
    fn init(&mut self) -> Vec<Task<Msg>> {
        vec![]
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::Click(mouse_event) => {
                self.clicked = true;
                if self.feature.sound {
                    if let Some(audio) = &self.click_audio {
                        let promise = audio.play().expect("must play");
                        spawn_local(async move {
                            JsFuture::from(promise).await.expect("must not error");
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
            Msg::ClickAudioMounted(node) => {
                let audio: HtmlAudioElement = node.unchecked_into();
                self.click_audio = Some(audio);
                Effects::none()
            }
            Msg::FrameMsg(fmsg) => {
                let effects =
                    <Frame<Msg> as Container<frame::Msg<Msg>, Msg>>::update(&mut self.frame, *fmsg);
                let (local, external) = effects.unzip();
                Effects::with_local(
                    local
                        .into_iter()
                        .map(|bmsg| Msg::FrameMsg(Box::new(bmsg)))
                        .chain(external),
                )
            }
            Msg::ChippedButtonMounted(me) => {
                let chipped_button: web_sys::Element = me.target_node.unchecked_into();
                self.chipped_button = Some(chipped_button);
                self.calc_button_dimension();
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
                    ("clicked", self.clicked),
                    ("click_highlights", self.feature.click_highlights),
                    ("expand_corners", self.feature.expand_corners),
                    ("has_corner_box_shadow", self.feature.has_corner_box_shadow),
                    ("has_underline", self.feature.has_underline),
                    ("hovered", self.hovered),
                    ("skewed", self.feature.skewed),
                    ("chipped", self.feature.chipped),
                    // setting this will also disable the div, therefore will not activate the
                    // events on it
                    ("disabled", self.feature.disabled),
                    ("hidden", self.feature.hidden),
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
            [
                audio(
                    [
                        on_mount(|me| Msg::ClickAudioMounted(me.target_node)),
                        src(&self.click_audio_src),
                    ],
                    [],
                ),
                <Frame<Msg> as Container<frame::Msg<Msg>, Msg>>::view(
                    &self.frame,
                    [
                        view_if(
                            self.feature.has_underline,
                            div([class_ns("underline underline-bottom")], []),
                        ),
                        self.view_button(),
                    ],
                )
                .map_msg(|fmsg| Msg::FrameMsg(Box::new(fmsg))),
            ],
        )
    }

    fn style(&self) -> Vec<String> {
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

        [vec![main], self.frame.style()].concat()
    }
}

impl<XMSG> Button<XMSG> {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme.clone();
        self.frame.set_theme(theme);
    }
    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
        self.frame.set_status(status);
    }

    fn calc_button_dimension(&mut self) {
        if let Some(chipped_button) = &self.chipped_button {
            let rect = chipped_button.get_bounding_client_rect();
            self.button_width = Some(rect.width() as f32);
            self.button_height = Some(rect.height() as f32);
        }
    }
}

impl<XMSG> Button<XMSG>
where
    XMSG: 'static,
{
    pub fn with_options(mut self, feature: Feature) -> Self {
        self.feature = feature;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.set_theme(theme);
        self
    }

    pub fn add_click_listener<F>(&mut self, f: F)
    where
        F: Fn(MouseEvent) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.click_listeners.push(cb);
    }

    ///transition time for most effects on the button
    fn transition_time_ms(&self) -> usize {
        250
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
            has_corner_box_shadow: false,
            has_underline: true,
            disabled: false,
            hidden: false,
            chipped: false,
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
            has_underline: false,
            ..Default::default()
        }
    }

    ///does not interact
    #[allow(unused)]
    pub fn disabled() -> Self {
        Feature {
            sound: false,
            click_highlights: false,
            has_corners: false,
            has_borders: true,
            expand_corners: false,
            has_underline: false,
            disabled: true,
            ..Default::default()
        }
    }
}

#[web_component]
impl<XMSG> CustomElement<Msg> for Button<XMSG>
where
    XMSG: 'static,
{
    fn custom_tag() -> &'static str {
        "sfui-button"
    }
    /// what attributes this component is interested in
    fn observed_attributes() -> Vec<&'static str> {
        vec![
            "label",
            "theme-primary",
            "theme-background",
            "feature",
            "status",
        ]
    }

    /// called when any of the attributes in observed_attributes is changed
    fn attribute_changed(
        program: &Program<Self, Msg>,
        attr_name: &str,
        _old_value: JsValue,
        new_value: JsValue,
    ) {
        let mut app = program.app.borrow_mut();
        match attr_name {
            "label" => {
                if let Some(label) = new_value.as_string() {
                    app.label = label;
                }
            }
            "width" => {
                if let Some(v) = new_value.as_f64() {
                    app.width = Some(v as i32);
                }
            }
            "height" => {
                if let Some(v) = new_value.as_f64() {
                    app.height = Some(v as i32);
                }
            }
            "theme-primary" => {
                if let Some(primary) = new_value.as_string() {
                    let background = &app.theme.background_color;
                    let theme =
                        Theme::from_str(&primary, background).expect("must be a valid theme");
                    app.set_theme(theme);
                }
            }
            "theme-background" => {
                if let Some(background) = new_value.as_string() {
                    let primary = &app.theme.primary_color;
                    let theme =
                        Theme::from_str(primary, &background).expect("must be a valid theme");
                    app.set_theme(theme);
                }
            }
            "feature" => {
                if let Some(v) = new_value.as_string() {
                    if let Ok(feature) = Feature::from_str(&v) {
                        app.set_feature(feature);
                    }
                }
            }
            "status" => {
                if let Some(v) = new_value.as_string() {
                    if let Ok(status) = Status::from_str(&v) {
                        app.set_status(status);
                    }
                }
            }
            _ => (),
        }
    }

    fn connected_callback(&mut self) {}
    fn disconnected_callback(&mut self) {}
    fn adopted_callback(&mut self) {}
}
