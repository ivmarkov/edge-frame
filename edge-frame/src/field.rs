use std::{cell::RefCell, rc::Rc};

use web_sys::Event;
use yew::{Callback, UseStateHandle};

use super::util::*;

#[derive(Clone)]
pub struct Field<R, S> {
    model_raw_value: Option<R>,
    raw_value: Rc<RefCell<Option<R>>>,
    value_state: UseStateHandle<Option<R>>,
    converter: Callback<Event, R>,
    validator: Callback<R, Result<S, String>>,
}

pub type TextField<S> = Field<String, S>;
pub type CheckedField<S> = Field<bool, S>;

impl<S> Field<String, S>
where
    S: Clone,
{
    pub fn text(
        model_raw_value: String,
        value_state: UseStateHandle<Option<String>>,
        validator: impl Fn(String) -> Result<S, String> + 'static,
    ) -> Rc<Self> {
        Rc::new(Self::new(
            Some(model_raw_value),
            value_state,
            get_input_text,
            validator,
        ))
    }
}

impl<S> Field<bool, S>
where
    S: Clone,
{
    pub fn checked(
        model_raw_value: bool,
        value_state: UseStateHandle<Option<bool>>,
        validator: impl Fn(bool) -> Result<S, String> + 'static,
    ) -> Rc<Self> {
        Rc::new(Self::new(
            Some(model_raw_value),
            value_state,
            get_input_checked,
            validator,
        ))
    }
}

impl<R, S> Field<R, S>
where
    R: Default + Clone + PartialEq + 'static,
    S: Clone,
{
    pub fn new(
        model_raw_value: Option<R>,
        value_state: UseStateHandle<Option<R>>,
        converter: impl Fn(Event) -> R + 'static,
        validator: impl Fn(R) -> Result<S, String> + 'static,
    ) -> Self {
        Self {
            model_raw_value,
            value_state,
            raw_value: Rc::new(RefCell::new(None)),
            converter: Callback::from(converter),
            validator: Callback::from(validator),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.has_errors()
            || self.raw_value.borrow_mut().is_some()
            || self.value_state.is_some() && *self.value_state != self.model_raw_value
    }

    pub fn value(&self) -> Option<S> {
        self.validator.emit(self.raw_value()).ok()
    }

    pub fn raw_value(&self) -> R {
        self.raw_value.borrow().clone().unwrap_or_else(|| {
            self.value_state.as_ref().cloned().unwrap_or_else(|| {
                self.model_raw_value
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
            this.on_change(event.into());
            callback.emit(());
        }
    }

    pub fn on_change(&self, event: Event) {
        let value = self.converter.emit(event);

        *self.raw_value.borrow_mut() = Some(value.clone());
        self.value_state.set(Some(value));
    }
}
