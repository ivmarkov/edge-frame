use core::cell::RefCell;
use core::fmt::Debug;

use std::rc::Rc;

use web_sys::Event;
use yew::{Callback, UseStateHandle};

use super::util::*;

#[derive(Clone)]
pub struct Field<R, S> {
    initial_value: Option<R>,
    raw_value: Rc<RefCell<Option<R>>>,
    value_state: UseStateHandle<Option<R>>,
    converter: Callback<Event, R>,
    validator: Callback<R, Result<S, String>>,
}

pub type TextField<S> = Field<String, S>;
pub type CheckedField<S> = Field<bool, S>;

impl<S> Field<String, S>
where
    S: Clone + Debug,
{
    pub fn text(
        initial_value: String,
        value_state: UseStateHandle<Option<String>>,
        validator: impl Fn(String) -> Result<S, String> + 'static,
    ) -> Rc<Self> {
        Rc::new(Self::new(
            Some(initial_value),
            value_state,
            get_input_text,
            validator,
        ))
    }
}

impl<S> Field<bool, S>
where
    S: Clone + Debug,
{
    pub fn checked(
        initial_value: bool,
        value_state: UseStateHandle<Option<bool>>,
        validator: impl Fn(bool) -> Result<S, String> + 'static,
    ) -> Rc<Self> {
        Rc::new(Self::new(
            Some(initial_value),
            value_state,
            get_input_checked,
            validator,
        ))
    }
}

impl<R, S> Field<R, S>
where
    R: Default + Clone + PartialEq + Debug + 'static,
    S: Clone + Debug,
{
    pub fn new(
        initial_value: Option<R>,
        value_state: UseStateHandle<Option<R>>,
        converter: impl Fn(Event) -> R + 'static,
        validator: impl Fn(R) -> Result<S, String> + 'static,
    ) -> Self {
        Self {
            initial_value,
            value_state,
            raw_value: Rc::new(RefCell::new(None)),
            converter: Callback::from(converter),
            validator: Callback::from(validator),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.has_errors()
            || self.raw_value.borrow_mut().is_some()
            || self.value_state.is_some() && *self.value_state != self.initial_value
    }

    pub fn value(&self) -> Option<S> {
        self.validator.emit(self.raw_value()).ok()
    }

    pub fn raw_value(&self) -> R {
        self.raw_value.borrow().clone().unwrap_or_else(|| {
            self.value_state.as_ref().cloned().unwrap_or_else(|| {
                self.initial_value
                    .clone()
                    .unwrap_or_else(|| Default::default())
            })
        })
    }

    pub fn has_errors(&self) -> bool {
        self.error().is_some()
    }

    pub fn error(&self) -> Option<String> {
        match self.validator.emit(self.raw_value()) {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }

    pub fn error_str(&self) -> String {
        self.error().unwrap_or_else(|| "\u{00a0}".into())
    }

    pub fn change<V>(&self, callback: Callback<()>) -> impl Fn(V)
    where
        V: Into<Event>,
    {
        let this = (*self).clone();

        move |event| {
            this.do_change(event.into(), &callback);
        }
    }

    pub fn do_change(&self, event: Event, callback: &Callback<()>) {
        let value = self.converter.emit(event);

        self.do_update(value, callback);
    }

    pub fn update<V>(&self, raw_value: R, callback: Callback<()>) -> impl Fn(V) {
        let this = (*self).clone();

        move |_| {
            this.do_update(raw_value.clone(), &callback);
        }
    }

    pub fn do_update(&self, value: R, callback: &Callback<()>) {
        log::debug!("Updating with {:?}", value);

        *self.raw_value.borrow_mut() = Some(value.clone());
        self.value_state.set(Some(value));

        callback.emit(());
    }
}
