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
const DEFAULT_WIDTH: usize = 100;
const DEFAULT_HEIGHT: usize = 40;

#[derive(Clone, Debug)]
pub enum Msg<XMSG> {
    Click(MouseEvent),
    HoverIn,
    HoverOut,
    HighlightEnd,
    External(XMSG),
    ContentTargetMounted(web_sys::Node),
}

#[derive(Debug)]
pub struct Frame<XMSG> {
    feature: Feature,
    clicked: bool,
    hovered: bool,
    click_listeners: Vec<Callback<MouseEvent, XMSG>>,
    width: Option<usize>,
    height: Option<usize>,
    theme: Theme,
    /// the status of the button which changes the color pallet of the button
    status: Option<Status>,
    children: Vec<Node<XMSG>>,
    content_target_node: Option<web_sys::Node>,
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

impl<XMSG> Default for Frame<XMSG>
where
    XMSG: 'static,
{
    fn default() -> Self {
        Self {
            feature: Feature::default(),
            clicked: false,
            hovered: false,
            click_listeners: vec![],
            width: None,
            height: None,
            theme: Theme::default(),
            status: None,
            children: vec![],
            content_target_node: None,
        }
    }
}

impl<XMSG> Frame<XMSG>
where
    XMSG: 'static,
{
    fn computed_width(&self) -> usize {
        // use the supplied width if it is specified
        if let Some(width) = self.width {
            width
        } else {
            DEFAULT_WIDTH
        }
    }

    fn computed_height(&self) -> usize {
        if let Some(height) = self.height {
            height
        } else {
            DEFAULT_HEIGHT
        }
    }

    fn view_borders(&self) -> Node<Msg<XMSG>> {
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

    fn view_corners(&self) -> Node<Msg<XMSG>> {
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
impl<XMSG> Container<Msg<XMSG>, XMSG> for Frame<XMSG>
where
    XMSG: 'static,
{
    fn update(&mut self, msg: Msg<XMSG>) -> Effects<Msg<XMSG>, XMSG> {
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
            Msg::ContentTargetMounted(target_node) => {
                log::debug!("target node is now mounted..");
                self.content_target_node = Some(target_node);
                Effects::none()
            }
        }
    }

    fn view(&self, content: impl IntoIterator<Item = Node<XMSG>>) -> Node<Msg<XMSG>> {
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
                        [on_mount(|me| Msg::ContentTargetMounted(me.target_node))],
                        content
                            .into_iter()
                            .chain(self.children.clone().into_iter())
                            .map(|node| node.map_msg(Msg::External)),
                    ),
                ],
            )],
        )
    }

    fn append_child(&mut self, child: Node<XMSG>) {
        self.children.push(child)
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

impl<XMSG> Frame<XMSG>
where
    XMSG: 'static,
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
        F: Fn(MouseEvent) -> XMSG + 'static,
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

impl<XMSG> CustomElement for Frame<XMSG> {
    /// what attributes this component is interested in
    fn observed_attributes() -> Vec<&'static str> {
        vec!["theme-primary", "theme-background", "feature", "status"]
    }

    /// called when any of the attributes in observed_attributes is changed
    fn attributes_changed(&mut self, attributes_values: BTreeMap<String, String>) {
        for (attribute, value) in attributes_values {
            match attribute.as_ref() {
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
                "status" => self.status = Status::from_str(value.as_ref()),
                _ => (),
            }
        }
    }
}

#[wasm_bindgen]
pub struct FrameCustomElement {
    program: Program<Frame<()>, Msg<()>>,
    children: Vec<web_sys::Node>,
}

fn extract_children_nodes(node: &web_sys::Node) -> Vec<web_sys::Node> {
    let node_list = node.child_nodes();
    let children_len = node_list.length() as usize;
    (0..children_len)
        .into_iter()
        .map(|i| node_list.item(i as u32).expect("must have an item"))
        .collect()
}

#[wasm_bindgen]
impl FrameCustomElement {
    #[wasm_bindgen(constructor)]
    pub fn new(node: JsValue) -> Self {
        log::info!("in constructor..");
        use sauron::wasm_bindgen::JsCast;

        let element_node: &web_sys::Element = node.unchecked_ref();
        let mount_node: &web_sys::Node = node.unchecked_ref();
        let children = extract_children_nodes(mount_node);
        log::info!("children: {:#?}", children);
        let outer_html = element_node.outer_html();
        log::debug!("outer html: {:#?}", outer_html);
        Self {
            program: Program::new(Frame::<()>::default(), mount_node, false, true),
            children,
        }
    }

    #[wasm_bindgen(getter, static_method_of = Self, js_name = observedAttributes)]
    pub fn observed_attributes() -> JsValue {
        let attributes = Frame::<Msg<()>>::observed_attributes();
        JsValue::from_serde(&attributes).expect("must be serde")
    }

    #[wasm_bindgen(method, js_name = attributeChangedCallback)]
    pub fn attribute_changed_callback(&self) {
        use sauron::wasm_bindgen::JsCast;
        use std::ops::DerefMut;

        log::debug!("attreibute changed callback");
        let mount_node = self.program.mount_node();
        let mount_element: &web_sys::Element = mount_node.unchecked_ref();
        let attribute_names = mount_element.get_attribute_names();
        let len = attribute_names.length();
        let mut attribute_values: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        for i in 0..len {
            let name = attribute_names.get(i);
            let attr_name = name.as_string().expect("must be a string attribute");
            if let Some(attr_value) = mount_element.get_attribute(&attr_name) {
                attribute_values.insert(attr_name, attr_value);
            }
        }
        self.program
            .app
            .borrow_mut()
            .deref_mut()
            .attributes_changed(attribute_values);
    }

    #[wasm_bindgen(method, js_name = connectedCallback)]
    pub fn connected_callback(&mut self) {
        use std::ops::Deref;

        log::info!("connected callback..");
        self.program.mount();
        let component_style =
            <Frame<()> as Application<Msg<()>>>::style(&self.program.app.borrow());
        self.program.inject_style_to_mount(&component_style);
        self.program.update_dom();
        self.append_children_to_shadow_mount();
    }

    //TODO: the best time to append the children to the shadown target mount
    //it to wait for the content target node to be mounted
    //once mounted, we mount this objects
    //
    // We can add a target_mount event listener to the frame
    // and the callback calls this append_children_to_shadow_mount
    fn append_children_to_shadow_mount(&self) {
        let mount_element: web_sys::Element = self.program.mount_node().unchecked_into();
        let mount_shadow = mount_element.shadow_root().expect("must have a shadow");
        let mount_shadow_node: &web_sys::Node = mount_shadow.unchecked_ref();
        for child in self.children.iter() {
            /*
            let target_node = self
                .program
                .app
                .borrow()
                .content_target_node
                .as_ref()
                .unwrap_or_else(|| {
                    let message = "must have a target node";
                    log::error!("{}", message);
                    panic!("{}", message);
                })
                .append_child(child)
                .expect("must append child");
            */
            mount_shadow_node
                .append_child(child)
                .expect("must append child..");
        }
    }

    #[wasm_bindgen(method, js_name = disconnectedCallback)]
    pub fn disconnected_callback(&mut self) {}

    #[wasm_bindgen(method, js_name = adoptedCallback)]
    pub fn adopted_callback(&mut self) {}

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_child(&mut self, child: JsValue) {
        use sauron::wasm_bindgen::JsCast;

        let child_node: web_sys::Node = child.unchecked_into();
        log::info!("a child is being appended: {:?}", child_node);
        log::info!("a child is being appended: {:?}", child_node.text_content());
        self.children.push(child_node);
    }
}

pub fn register() {
    sauron::register_custom_element("sfui-frame", "FrameCustomElement", "HTMLElement");
}
