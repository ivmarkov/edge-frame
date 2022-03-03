use std::{cell::RefCell, rc::Rc};

use web_sys::Event;
use yew::{use_state, Callback, UseStateHandle};

use crate::lambda::Lambda;

use super::util::*;

#[derive(Clone)]
pub struct Field<R, S> {
    value_state: UseStateHandle<R>,
    raw_value: Rc<RefCell<Option<R>>>,
    conv: Lambda<Event, R>,
    validate: Lambda<R, Result<S, String>>,
    changed: Callback<Result<S, String>>,
}

pub type TextField<S> = Field<String, S>;
pub type CheckedField<S> = Field<bool, S>;

impl<S> Field<String, S>
where
    S: Clone,
{
    pub fn text(
        validate: impl Fn(String) -> Result<S, String> + 'static,
        changed: impl Fn(Result<S, String>) + 'static,
    ) -> Self {
        Self::new(get_input_text, validate, changed)
    }
}

impl<S> Field<bool, S>
where
    S: Clone,
{
    pub fn checked(
        validate: impl Fn(bool) -> Result<S, String> + 'static,
        changed: impl Fn(Result<S, String>) + 'static,
    ) -> Self {
        Self::new(get_input_checked, validate, changed)
    }
}

impl<R, S> Field<R, S>
where
    R: Default + Clone + 'static,
    S: Clone,
{
    pub fn new(
        conv: impl Fn(Event) -> R + 'static,
        validate: impl Fn(R) -> Result<S, String> + 'static,
        changed: impl Fn(Result<S, String>) + 'static,
    ) -> Self {
        Self {
            value_state: use_state(|| Default::default()),
            raw_value: Rc::new(RefCell::new(None)),
            conv: Lambda::from(conv),
            validate: Lambda::from(validate),
            changed: Callback::from(changed),
        }
    }

    pub fn reset(&self, raw_value: R) {
        *self.raw_value.borrow_mut() = Some(raw_value.clone());
        self.value_state.set(raw_value);
    }

    pub fn value(&self) -> Option<S> {
        self.validate.call(self.raw_value()).ok()
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
        match self.validate.call(self.raw_value()) {
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
        self.update(self.conv.call(event));
    }

    fn update(&self, value: R) {
        self.reset(value);
        self.changed.emit(self.validate.call(self.raw_value()));
    }
}
