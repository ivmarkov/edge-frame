use std::{cell::RefCell, iter::once, mem, rc::Rc, str::FromStr};

//use anyhow::*;

use strum::{EnumMessage, IntoEnumIterator};

use yew::prelude::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Copy, Clone)]
pub struct Optional<T>(pub Option<T>);

impl<T> Optional<T> {
    pub fn new(v: Option<T>) -> Self {
        Self(v)
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(o: Option<T>) -> Self {
        Self(o)
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(o: Optional<T>) -> Self {
        o.0
    }
}

impl<T: FromStr> FromStr for Optional<T> {
    type Err = T::Err;

    fn from_str(mut s: &str) -> std::result::Result<Self, Self::Err> {
        s = s.trim();

        if s.len() == 0 {
            Ok(Optional(None))
        } else {
            s.parse::<T>().map(|op| Optional(Some(op)))
        }
    }
}

impl<T: ToString> ToString for Optional<T> {
    fn to_string(&self) -> String {
        match self.0.as_ref() {
            None => "".into(),
            Some(v) => v.to_string(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Loadable<T> {
    Not,
    Loading(Option<T>),
    Loaded(T),
}

impl<T> Loadable<T> {
    pub fn is_loaded(&self) -> bool {
        match self {
            Self::Loaded(_) | Self::Loading(Some(_)) => true,
            _ => false,
        }
    }

    pub fn is_loading(&self) -> bool {
        match self {
            Self::Loading(_) => true,
            _ => false,
        }
    }

    pub fn loading(&mut self) {
        let old_self = mem::replace(self, Loadable::Not);
        match old_self {
            Self::Not => *self = Self::Loading(None),
            Self::Loaded(data) => *self = Self::Loading(Some(data)),
            _ => (),
        }
    }

    pub fn loaded(&mut self, data: T) {
        *self = Self::Loaded(data)
    }

    pub fn loaded_result(&mut self, result: Result<T>) {
        match result {
            Ok(data) => *self = Self::Loaded(data),
            Err(_) => {
                let old_self = mem::replace(self, Loadable::Not);
                match old_self {
                    Self::Loading(Some(data)) => *self = Self::Loaded(data),
                    other => *self = other,
                }
            }
        }
    }

    pub fn data_ref(&self) -> Option<&T> {
        if let Loadable::Loaded(data) = self {
            Some(data)
        } else if let Loadable::Loading(Some(data)) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn try_data_mut(&mut self) -> Option<&mut T> {
        if let Loadable::Loaded(data) = self {
            Some(data)
        } else if let Loadable::Loading(Some(data)) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn data_mut(&mut self) -> &mut T {
        self.try_data_mut().unwrap()
    }
}

impl<T> Default for Loadable<T> {
    fn default() -> Self {
        Self::Not
    }
}

#[derive(Copy, Clone)]
pub struct Editable<T> {
    changed: bool,
    data: T,
}

impl<T> Editable<T> {
    pub fn new(data: T) -> Self {
        Self {
            changed: false,
            data,
        }
    }

    pub fn get(self) -> T {
        self.data
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}

impl<T: Default> Default for Editable<T> {
    fn default() -> Self {
        Editable::new(Default::default())
    }
}

impl<T> AsRef<T> for Editable<T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T> AsMut<T> for Editable<T> {
    fn as_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.data
    }
}

#[derive(Clone)]
pub struct Model<T>(pub Rc<RefCell<Loadable<T>>>);

impl<T> Default for Model<T> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Loadable::Not)))
    }
}

impl<T> Model<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

pub trait Description {
    fn get_description(&self) -> String;
}

pub trait IntoDomainIterator: Sized {
    type Iterator: Iterator<Item = Self>;

    fn iter() -> Self::Iterator;
}

impl<T: EnumMessage> Description for T {
    fn get_description(&self) -> String {
        self.get_message().map_or("".into(), |v| v.into())
    }
}

impl<I: Iterator<Item = T>, T: IntoEnumIterator<Iterator = I>> IntoDomainIterator for T {
    type Iterator = <T as IntoEnumIterator>::Iterator;

    fn iter() -> Self::Iterator {
        <T as IntoEnumIterator>::iter()
    }
}

pub struct Field<T> {
    string: String,
    error: Option<String>,
    parser: Box<dyn Fn(&str) -> Result<T>>,
    stringifier: Box<dyn Fn(T) -> String>,
    getter: Box<dyn Fn() -> Option<T>>,
    updater: Box<dyn Fn(T)>,
}

impl<T: FromStr + ToString + 'static> Default for Field<T> {
    fn default() -> Self {
        Self::new(
            |s: &str| {
                s.trim()
                    .parse::<T>()
                    .map_err(|_| Error::msg("Invalid format"))
            },
            |t| t.to_string(),
        )
    }
}

impl<T: 'static> Field<T> {
    pub fn new(
        parser: impl Fn(&str) -> Result<T> + 'static,
        stringifier: impl Fn(T) -> String + 'static,
    ) -> Self {
        Self {
            string: Default::default(),
            error: None,
            parser: Box::new(parser),
            stringifier: Box::new(stringifier),
            getter: Box::new(|| None),
            updater: Box::new(|_| {}),
        }
    }

    pub fn bind(
        &mut self,
        getter: impl Fn() -> Option<T> + 'static,
        updater: impl Fn(T) + 'static,
    ) {
        self.getter = Box::new(getter);
        self.updater = Box::new(updater);

        self.load();
    }

    pub fn load(&mut self) {
        self.error = None;
        self.string = (self.getter)().map_or("".to_owned(), |d| (self.stringifier)(d));
    }

    pub fn update(&mut self, value: String) {
        let value = value.trim();

        if self.string != value {
            self.string = value.to_owned();

            match (self.parser)(value) {
                Ok(data) => {
                    self.error = None;
                    (self.updater)(data);
                }
                Err(err) => {
                    self.error = Some(err.to_string());
                }
            }
        }
    }

    pub fn get_value_str(&self) -> &str {
        self.string.as_ref()
    }

    pub fn update_value(&self) -> &str {
        self.string.as_ref()
    }

    pub fn get_value(&self) -> Option<T> {
        (self.parser)(self.string.as_ref()).ok()
    }

    pub fn is_valid(&self) -> bool {
        match &self.error {
            None => true,
            Some(_) => false,
        }
    }

    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }

    pub fn get_error_str(&self) -> String {
        self.error.clone().unwrap_or("".to_owned())
    }

    pub fn get_description(v: T) -> String
    where
        T: Description,
    {
        v.get_description()
    }

    pub fn get_domain(&self) -> Vec<T>
    where
        T: IntoDomainIterator,
    {
        T::iter().collect()
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct CenteredGridProps {
    #[prop_or_default]
    pub children: Children,
}

#[derive(Debug)]
pub struct CenteredGrid {
    props: CenteredGridProps,
}

pub enum CenteredGridMsg {}

impl Component for CenteredGrid {
    type Message = CenteredGridMsg;
    type Properties = CenteredGridProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        CenteredGrid { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="page">
                <div class="mdc-layout-grid">
                    <div class="mdc-layout-grid__inner">
                        // Spacer
                        <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-3 mdc-layout-grid__cell--span-2-tablet mdc-layout-grid__cell--span-1-phone"></div>

                        // Content
                        <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-6 mdc-layout-grid__cell--span-4-tablet mdc-layout-grid__cell--span-2-phone">
                        { self.props.children.clone() }
                        </div>

                        // Spacer
                        <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-3 mdc-layout-grid__cell--span-2-tablet mdc-layout-grid__cell--span-1-phone"></div>
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Left,
    Right,
    Center,
}

impl Default for Align {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
    Stretch,
}

impl Default for VAlign {
    fn default() -> Self {
        Self::Stretch
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct GridProps {
    #[prop_or_default]
    pub inner: bool,

    #[prop_or_default]
    pub align: Align,

    #[prop_or_default]
    pub children: Children,
}

#[derive(Debug)]
pub struct Grid {
    props: GridProps,
}

pub enum GridMsg {}

impl Component for Grid {
    type Message = GridMsg;
    type Properties = GridProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Grid { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let grid = html! {
            <div class="mdc-layout-grid__inner">
                { self.props.children.clone() }
            </div>
        };

        if !self.props.inner {
            let classes = classes!(
                "mdc-layout-grid",
                match self.props.align {
                    Align::Left => Some("left"),
                    Align::Right => Some("right"),
                    Align::Center => None,
                }
                .map(|align| format!("mdc-layout-grid--align-{}", align))
            );

            html! {
                <div class=classes>
                    { grid }
                </div>
            }
        } else {
            grid
        }
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct CellProps {
    #[prop_or_default]
    pub valign: VAlign,

    #[prop_or_default]
    pub order: Option<u32>,

    #[prop_or_default]
    pub span: Option<u32>,

    #[prop_or_default]
    pub span_desktop: Option<u32>,

    #[prop_or_default]
    pub span_tablet: Option<u32>,

    #[prop_or_default]
    pub span_phone: Option<u32>,

    #[prop_or_default]
    pub style: String,

    #[prop_or_default]
    pub children: Children,
}

pub enum CellMsg {}

#[derive(Debug)]
pub struct Cell {
    props: CellProps,
}

impl Component for Cell {
    type Message = CellMsg;
    type Properties = CellProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Cell { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let classes = once("mdc-layout-grid__cell".into())
            .chain(
                [
                    ("", self.props.span),
                    ("-desktop", self.props.span_desktop),
                    ("-tablet", self.props.span_tablet),
                    ("-phone", self.props.span_phone),
                ]
                .iter()
                .filter_map(|(prefix, value)| {
                    value.map(|span| format!("mdc-layout-grid__cell--span-{}{}", span, prefix))
                }),
            )
            .chain(
                match self.props.valign {
                    VAlign::Top => Some("top"),
                    VAlign::Middle => Some("middle"),
                    VAlign::Bottom => Some("bottom"),
                    VAlign::Stretch => None,
                }
                .map(|valign| format!("mdc-layout-grid__cell--align-{}", valign))
                .into_iter(),
            )
            .chain(
                self.props
                    .order
                    .map(|order| format!("mdc-layout-grid__cell--order-{}", order))
                    .into_iter(),
            )
            .collect::<Vec<_>>();

        html! {
            <div class=classes!(classes) style=self.props.style.clone()>
                { self.props.children.clone() }
            </div>
        }
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct ChunkProps {
    #[prop_or(true)]
    pub visible: bool,

    #[prop_or_default]
    pub children: Children,
}

pub enum ChunkMsg {}

#[derive(Debug)]
pub struct Chunk {
    props: ChunkProps,
}

impl Component for Chunk {
    type Message = ChunkMsg;
    type Properties = ChunkProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Chunk { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        if self.props.visible {
            html! {
                <>
                { self.props.children.clone() }
                </>
            }
        } else {
            html! {}
        }
    }
}
