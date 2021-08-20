//! This module contains data types for interacting with `Scope`s.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// Universal closure wrapper.
/// <aside class="warning">
/// Use closure adapters carefully, because if you call one from the `update` loop
/// of a `Component` (even from JS) it will delay a message until next.
/// Closure adapters should be used from JS closure adapters or `setTimeout` calls.
/// </aside>
/// An `Rc` wrapper is used to make it cloneable.
pub enum Lambda<IN, OUT> {
    /// A closure which can be called multiple times
    Lambda(Rc<dyn Fn(IN) -> OUT>),
    /// A closure which can only be called once. The closure will panic if it is
    /// called more than once.
    LambdaOnce(Rc<LambdaOnce<IN, OUT>>),
}

type LambdaOnce<IN, OUT> = RefCell<Option<Box<dyn FnOnce(IN) -> OUT>>>;

impl<IN, OUT, F: Fn(IN) -> OUT + 'static> From<F> for Lambda<IN, OUT> {
    fn from(func: F) -> Self {
        Lambda::Lambda(Rc::new(func))
    }
}

impl<IN, OUT> Clone for Lambda<IN, OUT> {
    fn clone(&self) -> Self {
        match self {
            Lambda::Lambda(cb) => Lambda::Lambda(cb.clone()),
            Lambda::LambdaOnce(cb) => Lambda::LambdaOnce(cb.clone()),
        }
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<IN, OUT> PartialEq for Lambda<IN, OUT> {
    fn eq(&self, other: &Lambda<IN, OUT>) -> bool {
        match (&self, &other) {
            (Lambda::Lambda(cb), Lambda::Lambda(other_cb)) => Rc::ptr_eq(cb, other_cb),
            (Lambda::LambdaOnce(cb), Lambda::LambdaOnce(other_cb)) => Rc::ptr_eq(cb, other_cb),
            _ => false,
        }
    }
}

impl<IN, OUT> fmt::Debug for Lambda<IN, OUT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            Lambda::Lambda(_) => "Lambda<_>",
            Lambda::LambdaOnce(_) => "LambdaOnce<_>",
        };

        f.write_str(data)
    }
}

impl<IN, OUT> Lambda<IN, OUT> {
    /// This method calls the closure's function.
    pub fn call(&self, value: IN) -> OUT {
        match self {
            Lambda::Lambda(cb) => cb(value),
            Lambda::LambdaOnce(rc) => {
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
        Lambda::LambdaOnce(Rc::new(RefCell::new(Some(Box::new(func)))))
    }
}

impl<IN, OUT: Default> Lambda<IN, OUT> {
    /// Creates a "no-op" closure which can be used when it is not suitable to use an
    /// `Option<Lambda>`.
    pub fn noop() -> Self {
        Self::from(|_| -> OUT { Default::default() })
    }
}

impl<IN, OUT: Default> Default for Lambda<IN, OUT> {
    fn default() -> Self {
        Self::noop()
    }
}

impl<IN: 'static, OUT: 'static> Lambda<IN, OUT> {
    /// Changes the input & output types of the closure to others.
    /// Works like the `map` method but in the opposite direction.
    pub fn reform<FI, FO, A, R>(&self, map_input: FI, map_output: FO) -> Lambda<A, R>
    where
        FI: Fn(A) -> IN + 'static,
        FO: Fn(OUT) -> R + 'static,
    {
        let this = self.clone();
        let func = move |input| map_output(this.call(map_input(input)));
        Lambda::from(func)
    }
}

// #[cfg(test)]
// pub(crate) mod test_util {
//     use super::*;
//     use std::cell::RefCell;
//     use std::future::Future;
//     use std::pin::Pin;
//     use std::task::{Context, Poll, Waker};

//     struct HOCHandle<T> {
//         waker: Option<Waker>,
//         output: Option<T>,
//     }

//     impl<T> Default for HOCHandle<T> {
//         fn default() -> Self {
//             HOCHandle {
//                 waker: None,
//                 output: None,
//             }
//         }
//     }

//     pub(crate) struct HOCFuture<T>(Rc<RefCell<HOCHandle<T>>>);

//     impl<T> Clone for HOCFuture<T> {
//         fn clone(&self) -> Self {
//             Self(self.0.clone())
//         }
//     }

//     impl<T> Default for HOCFuture<T> {
//         fn default() -> Self {
//             Self(Rc::default())
//         }
//     }

//     impl<T: 'static> Into<Lambda<T>> for HOCFuture<T> {
//         fn into(self) -> Lambda<T> {
//             Lambda::from(move |r| self.finish(r))
//         }
//     }

//     impl<T> Future for CallbackFuture<T> {
//         type Output = T;
//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             if let Some(output) = self.ready() {
//                 Poll::Ready(output)
//             } else {
//                 let handle = &self.0;
//                 handle.borrow_mut().waker = Some(cx.waker().clone());
//                 Poll::Pending
//             }
//         }
//     }

//     impl<T> CallbackFuture<T> {
//         fn ready(&self) -> Option<T> {
//             self.0.borrow_mut().output.take()
//         }

//         fn finish(&self, output: T) {
//             self.0.borrow_mut().output = Some(output);
//             if let Some(waker) = self.0.borrow_mut().waker.take() {
//                 waker.wake();
//             }
//         }
//     }
// }
