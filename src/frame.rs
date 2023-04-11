use crate::Status;
use crate::Theme;
use async_trait::async_trait;
use css_colors::Color;
use sauron::jss_ns_pretty;
use sauron::{
    dom::Callback,
    html::attributes,
    html::{attributes::class, div, events::on_click},
    prelude::*,
    Node,
};
use std::collections::BTreeMap;
use std::str::FromStr;
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
    ContentTargetMounted(MountEvent),
}

impl<XMSG> From<XMSG> for Msg<XMSG> {
    fn from(xmsg: XMSG) -> Self {
        Msg::External(xmsg)
    }
}

#[derive(Debug)]
pub struct Frame<XMSG> {
    feature: Feature,
    clicked: bool,
    hovered: bool,
    click_listeners: Vec<Callback<MouseEvent, XMSG>>,
    /// called when the container for the content is mounted
    container_mounted_listeners: Vec<Callback<MountEvent, XMSG>>,
    width: Option<usize>,
    height: Option<usize>,
    theme: Theme,
    /// the status of the button which changes the color pallet of the button
    status: Option<Status>,
    children: Vec<Node<XMSG>>,
    content_target_node: Option<web_sys::Node>,
    dimension: Dimension,
}

#[derive(Debug)]
pub struct Feature {
    /// has corners
    pub has_corners: bool,
    /// the frame has borders
    pub has_borders: bool,
    /// expand corners when hovered
    pub expand_corners: bool,
    pub has_corner_box_shadow: bool,
}

#[derive(Debug, Clone)]
pub struct Dimension {
    /// width of the corner clip of this button
    pub corner_width: i32,
    /// lengths of the corner clip of this button
    pub corner_length: i32,
    /// distance that clips at the corner expands when the button is hovered
    pub corner_expand_distance: i32,
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
            container_mounted_listeners: vec![],
            width: None,
            height: None,
            theme: Theme::default(),
            status: None,
            children: vec![],
            content_target_node: None,
            dimension: Dimension::default(),
        }
    }
}

impl<XMSG> Frame<XMSG>
where
    XMSG: 'static,
{
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_feature(&mut self, feature: Feature) {
        self.feature = feature;
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }
    fn computed_width(&self) -> usize {
        // use the supplied width if it is specified
        if let Some(width) = self.width {
            width
        } else {
            DEFAULT_WIDTH
        }
    }

    pub fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension;
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

#[async_trait(?Send)]
impl<XMSG> Container<Msg<XMSG>, XMSG> for Frame<XMSG>
where
    XMSG: 'static,
{
    async fn update(&mut self, msg: Msg<XMSG>) -> Effects<Msg<XMSG>, XMSG> {
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
            Msg::ContentTargetMounted(me) => {
                let mount_event = me.clone();
                let target_node = me.target_node;
                self.content_target_node = Some(target_node);
                let external = self
                    .container_mounted_listeners
                    .iter()
                    .map(|listener| listener.emit(mount_event.clone()));
                Effects::with_external(external)
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
                        [on_mount(|me| Msg::ContentTargetMounted(me))],
                        content
                            .into_iter()
                            .chain(self.children.clone().into_iter())
                            .map(|node| node.map_msg(|xmsg| Msg::from(xmsg))),
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

    pub fn add_click_listener<F>(&mut self, f: F)
    where
        F: Fn(MouseEvent) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.click_listeners.push(cb);
    }

    pub fn add_container_mounted_listener<F>(&mut self, f: F)
    where
        F: Fn(MountEvent) -> XMSG + 'static,
    {
        let cb = Callback::from(f);
        self.container_mounted_listeners.push(cb);
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
        let transition_time_ms = self.transition_time_ms();

        let Dimension {
            corner_width,
            corner_length,
            corner_expand_distance,
        } = self.dimension;

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
            has_corners: true,
            has_borders: true,
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
                "status" => self.status = Status::from_str(value.as_ref()).ok(),
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
        use sauron::wasm_bindgen::JsCast;

        let mount_node: &web_sys::Node = node.unchecked_ref();
        let children = extract_children_nodes(mount_node);
        Self {
            program: Program::new(Frame::<()>::default(), mount_node, true),
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
            .attributes_changed(attribute_values);
    }

    #[wasm_bindgen(method, js_name = connectedCallback)]
    pub fn connected_callback(&mut self) {
        use std::ops::DerefMut;
        self.program.mount();
        let component_style =
            <Frame<()> as Application<Msg<()>>>::style(&self.program.app.borrow());
        self.program.inject_style_to_mount(&component_style);
        self.program.update_dom();

        let children: Vec<web_sys::Node> = self.children.clone();
        self.program
            .app
            .borrow_mut()
            .add_container_mounted_listener(move |me| {
                Self::append_children_to_shadow_mount(me.target_node, children.clone());
            });
    }

    fn append_children_to_shadow_mount(target_node: web_sys::Node, children: Vec<web_sys::Node>) {
        for child in children.iter() {
            target_node
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
        self.children.push(child_node);
    }
}

pub fn register() {
    sauron::register_custom_element("sfui-frame", "FrameCustomElement", "HTMLElement");
}
