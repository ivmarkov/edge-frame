use std::{fmt::Debug, ops::Deref, ptr, rc::Rc};

use yew::{use_context, Reducible, UseReducerHandle};

pub struct Projection<S, P, A>
where
    S: Reducible,
{
    state_mapper: Rc<dyn Fn(&S) -> &P>,
    action_mapper: Rc<dyn Fn(A) -> S::Action>,
}

impl<S, P, A> Projection<S, P, A>
where
    S: Reducible,
{
    pub fn new(
        state_mapper: impl Fn(&S) -> &P + 'static,
        action_mapper: impl Fn(A) -> S::Action + 'static,
    ) -> Self {
        Self {
            state_mapper: Rc::new(state_mapper),
            action_mapper: Rc::new(action_mapper),
        }
    }
}

impl<S, P, A> Clone for Projection<S, P, A>
where
    S: Reducible,
{
    fn clone(&self) -> Self {
        Self {
            state_mapper: self.state_mapper.clone(),
            action_mapper: self.action_mapper.clone(),
        }
    }
}

impl<S, P, A> PartialEq for Projection<S, P, A>
where
    S: Reducible,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(
            &*self.state_mapper as *const _,
            &*other.state_mapper as *const _,
        ) && ptr::eq(
            &*self.action_mapper as *const _,
            &*other.action_mapper as *const _,
        )
    }
}

impl<S, P, A> Debug for Projection<S, P, A>
where
    S: Reducible,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Projection").finish()
    }
}

pub struct UseProjectionHandle<S, P, A>
where
    S: Reducible,
{
    projection: Projection<S, P, A>,
    reducer_handle: UseReducerHandle<S>,
}

impl<S, P, A> UseProjectionHandle<S, P, A>
where
    S: Reducible,
{
    pub fn dispatch(&self, action: A) {
        self.reducer_handle
            .dispatch((self.projection.action_mapper)(action))
    }
}

impl<S, P, A> Deref for UseProjectionHandle<S, P, A>
where
    S: Reducible,
{
    type Target = P;

    fn deref(&self) -> &Self::Target {
        (self.projection.state_mapper)(&*self.reducer_handle)
    }
}

pub fn use_projection<S: Store, P, A>(
    projection: Projection<S, P, A>,
) -> UseProjectionHandle<S, P, A> {
    let store = use_context::<StoreContext<S>>().expect("No Store context found");

    UseProjectionHandle {
        projection,
        reducer_handle: store.0,
    }
}

pub trait Store: Reducible + Clone + PartialEq + 'static
/*where <Self as Reducible>::Action: PartialEq + 'static*/
{
}

impl<T> Store for T
where
    T: Reducible + Clone + PartialEq + 'static,
    T::Action: PartialEq + 'static,
{
}

pub struct StoreContext<T: Store>(UseReducerHandle<T>);

impl<T: Store> StoreContext<T> {
    pub fn new(reducer_handle: UseReducerHandle<T>) -> Self {
        Self(reducer_handle)
    }
}

impl<T: Store> Clone for StoreContext<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Store> PartialEq for StoreContext<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct SimpleStore<S>(S);

impl<S> SimpleStore<S> {
    pub fn new(state: S) -> Self {
        Self(state)
    }
}

impl<S> Reducible for SimpleStore<S> {
    type Action = SimpleStoreAction<S>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let this = match action {
            Self::Action::Update(state) => Self(state),
        };

        this.into()
    }
}

impl<S> Deref for SimpleStore<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq)]
pub enum SimpleStoreAction<S> {
    Update(S),
}
