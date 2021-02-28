use std::{cell::RefCell, mem, rc::Rc, str::FromStr};

use anyhow::*;

use strum::{EnumMessage, IntoEnumIterator};

#[derive(Copy, Clone)]
pub struct Optional<T>(pub Option<T>);

impl<T: FromStr> FromStr for Optional<T> {
    type Err = T::Err;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
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
            Some(v) => v.to_string()
        }
    }
}

#[derive(Copy, Clone)]
pub enum Loadable<T> {
    Not,
    Loading(Option<T>),
    Loaded(T)
}

impl<T> Loadable<T> {
    pub fn is_loaded(&self) -> bool {
        match self {
            &Self::Loaded(_) => true,
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
        } else {
            None
        }
    }

    pub fn try_data_mut(&mut self) -> Option<&mut T> {
        if let Loadable::Loaded(data) = self {
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
    data: T
}

impl<T> Editable<T> {
    pub fn new(data: T) -> Self {
        Self {
            changed: false,
            data
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
pub struct Model<T>(pub Rc<RefCell<Loadable<Editable<T>>>>);

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

impl <T: IntoEnumIterator> IntoDomainIterator for T {
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
            |s: &str| s.trim().parse::<T>().map_err(|_| Error::msg("Invalid format")),
            |t| t.to_string())
    }
}

impl<T: 'static> Field<T> {
    pub fn new(
            parser: impl Fn(&str) -> Result<T> + 'static,
            stringifier: impl Fn(T) -> String + 'static) -> Self {
        Self {
            string: Default::default(),
            error: None,
            parser: Box::new(parser),
            stringifier: Box::new(stringifier),
            getter: Box::new(|| None),
            updater: Box::new(|_| {})
        }
    }

    pub fn bind(
            &mut self,
            getter: impl Fn() -> Option<T> + 'static,
            updater: impl Fn(T) + 'static) {
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
                },
                Err(err) => {
                    self.error = Some(err.to_string());
                }
            }
        }
    }

    pub fn get_value(&self) -> Option<T> {
        (self.parser)(self.string.as_ref()).ok()
    }

    pub fn get_value_str(&self) -> &str {
        self.string.as_ref()
    }

    pub fn is_valid(&self) -> bool {
        match &self.error {
            None => true,
            Some(_) => false
        }
    }

    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }

    pub fn get_error_str(&self) -> String {
        self.error.clone().unwrap_or("".to_owned())
    }

    pub fn get_description(v: T) -> String where T: Description {
        v.get_description()
    }

    pub fn get_domain(&self) -> Vec<T> where T: IntoDomainIterator {
        T::iter().collect()
    }
}
