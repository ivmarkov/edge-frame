//! This module contains data types for interacting with `Scope`s.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use yew::html::ImplicitClone;

/// Universal closure wrapper.
/// <aside class="warning">
/// Use closure adapters carefully, because if you call one from the `update` loop
/// of a `Component` (even from JS) it will delay a message until next.
/// Closure adapters should be used from JS closure adapters or `setTimeout` calls.
/// </aside>
/// An `Rc` wrapper is used to make it cloneable.
pub enum Callback2<IN, OUT> {
    /// A closure which can be called multiple times
    Callback2(Rc<dyn Fn(IN) -> OUT>),
    /// A closure which can only be called once. The closure will panic if it is
    /// called more than once.
    Callback2Once(Rc<Callback2Once<IN, OUT>>),
}

type Callback2Once<IN, OUT> = RefCell<Option<Box<dyn FnOnce(IN) -> OUT>>>;

impl<IN, OUT, F: Fn(IN) -> OUT + 'static> From<F> for Callback2<IN, OUT> {
    fn from(func: F) -> Self {
        Callback2::Callback2(Rc::new(func))
    }
}

impl<IN, OUT> Clone for Callback2<IN, OUT> {
    fn clone(&self) -> Self {
        match self {
            Callback2::Callback2(cb) => Callback2::Callback2(cb.clone()),
            Callback2::Callback2Once(cb) => Callback2::Callback2Once(cb.clone()),
        }
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<IN, OUT> PartialEq for Callback2<IN, OUT> {
    fn eq(&self, other: &Callback2<IN, OUT>) -> bool {
        match (&self, &other) {
            (Callback2::Callback2(cb), Callback2::Callback2(other_cb)) => Rc::ptr_eq(cb, other_cb),
            (Callback2::Callback2Once(cb), Callback2::Callback2Once(other_cb)) => {
                Rc::ptr_eq(cb, other_cb)
            }
            _ => false,
        }
    }
}

impl<IN, OUT> fmt::Debug for Callback2<IN, OUT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            Callback2::Callback2(_) => "Lambda<_>",
            Callback2::Callback2Once(_) => "LambdaOnce<_>",
        };

        f.write_str(data)
    }
}

impl<IN, OUT> Callback2<IN, OUT> {
    /// This method calls the closure's function.
    pub fn call(&self, value: IN) -> OUT {
        match self {
            Callback2::Callback2(cb) => cb(value),
            Callback2::Callback2Once(rc) => {
                let cb = rc.replace(None);
                let f = cb.expect("closure in LambdaOnce has already been used");
                f(value)
            }
        }
    }

    /// Creates a closure from an `FnOnce`. The programmer is responsible for ensuring
    /// that the closure is only called once. If it is called more than once, the closure
    /// will panic.
    pub fn once<F>(func: F) -> Self
    where
        F: FnOnce(IN) -> OUT + 'static,
    {
        Callback2::Callback2Once(Rc::new(RefCell::new(Some(Box::new(func)))))
    }
}

impl<IN, OUT: Default> Callback2<IN, OUT> {
    /// Creates a "no-op" closure which can be used when it is not suitable to use an
    /// `Option<Lambda>`.
    pub fn noop() -> Self {
        Self::from(|_| -> OUT { Default::default() })
    }
}

impl<IN, OUT: Default> Default for Callback2<IN, OUT> {
    fn default() -> Self {
        Self::noop()
    }
}

impl<IN: 'static, OUT: 'static> Callback2<IN, OUT> {
    /// Changes the input & output types of the closure to others.
    /// Works like the `map` method but in the opposite direction.
    pub fn reform<FI, FO, A, R>(&self, map_input: FI, map_output: FO) -> Callback2<A, R>
    where
        FI: Fn(A) -> IN + 'static,
        FO: Fn(OUT) -> R + 'static,
    {
        let this = self.clone();
        let func = move |input| map_output(this.call(map_input(input)));
        Callback2::from(func)
    }
}

impl<IN, OUT> ImplicitClone for Callback2<IN, OUT> {}
