use std::{cell::RefCell, rc::Rc};

use web_sys::Event;
use yew::{use_state, UseStateHandle};

use super::callback2::Callback2;
use super::*;

#[derive(Clone)]
pub struct Field<R, S> {
    value_state: UseStateHandle<R>,
    raw_value: Rc<RefCell<Option<R>>>,
    converter: Callback2<Event, R>,
    validator: Callback2<R, Result<S, String>>,
}

pub type TextField<S> = Field<String, S>;
pub type CheckedField<S> = Field<bool, S>;

impl<S> Field<String, S>
where
    S: Clone,
{
    pub fn text(validate: impl Fn(String) -> Result<S, String> + 'static) -> Self {
        Self::new(get_input_text, validate)
    }
}

impl<S> Field<bool, S>
where
    S: Clone,
{
    pub fn checked(validate: impl Fn(bool) -> Result<S, String> + 'static) -> Self {
        Self::new(get_input_checked, validate)
    }
}

impl<R, S> Field<R, S>
where
    R: Default + Clone + 'static,
    S: Clone,
{
    pub fn new(
        converter: impl Fn(Event) -> R + 'static,
        validate: impl Fn(R) -> Result<S, String> + 'static,
    ) -> Self {
        Self {
            value_state: use_state(|| Default::default()),
            raw_value: Rc::new(RefCell::new(None)),
            converter: Callback2::from(converter),
            validator: Callback2::from(validate),
        }
    }

    pub fn set(&self, raw_value: R) {
        *self.raw_value.borrow_mut() = Some(raw_value.clone());
        self.value_state.set(raw_value);
    }

    pub fn value(&self) -> Option<S> {
        self.validator.call(self.raw_value()).ok()
    }

    pub fn raw_value(&self) -> R {
        self.raw_value
            .borrow()
            .clone()
            .unwrap_or((&*self.value_state).clone())
    }

    pub fn has_errors(&self) -> bool {
        self.error().is_some()
    }

    pub fn error(&self) -> Option<String> {
        match self.validator.call(self.raw_value()) {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }

    pub fn error_str(&self) -> String {
        self.error().unwrap_or_else(|| "\u{00a0}".into())
    }

    pub fn change<V>(&self) -> impl Fn(V)
    where
        V: Into<Event>,
    {
        let this = (*self).clone();

        move |event| this.on_change(event.into())
    }

    pub fn on_change(&self, event: Event) {
        self.set(self.converter.call(event));
    }
}
