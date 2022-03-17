use std::{fmt::Debug, ops::Deref, ptr, rc::Rc};

use log::{log, Level};
use yew::{use_context, Reducible, UseReducerHandle};

pub trait StoreHandle: Deref {
    type Action;

    fn dispatch(&self, action: Self::Action);
}

impl<R> StoreHandle for UseReducerHandle<R>
where
    R: Reducible,
{
    type Action = R::Action;

    fn dispatch(&self, action: Self::Action) {
        UseReducerHandle::dispatch(self, action)
    }
}

pub struct Projection<R, P, A>
where
    R: Reducible,
{
    state_mapper: Rc<dyn Fn(&R) -> &P>,
    action_mapper: Rc<dyn Fn(A) -> R::Action>,
}

impl<R, P, A> Projection<R, P, A>
where
    R: Reducible,
{
    pub fn new(
        state_mapper: impl Fn(&R) -> &P + 'static,
        action_mapper: impl Fn(A) -> R::Action + 'static,
    ) -> Self {
        Self {
            state_mapper: Rc::new(state_mapper),
            action_mapper: Rc::new(action_mapper),
        }
    }
}

impl<R, P, A> Clone for Projection<R, P, A>
where
    R: Reducible,
{
    fn clone(&self) -> Self {
        Self {
            state_mapper: self.state_mapper.clone(),
            action_mapper: self.action_mapper.clone(),
        }
    }
}

impl<R, P, A> PartialEq for Projection<R, P, A>
where
    R: Reducible,
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

impl<R, P, A> Debug for Projection<R, P, A>
where
    R: Reducible,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Projection").finish()
    }
}

pub struct UseProjectionHandle<R, P, A>
where
    R: Reducible,
{
    projection: Projection<R, P, A>,
    store: Store<R>,
}

impl<R, P, A> UseProjectionHandle<R, P, A>
where
    R: Reducible,
{
    fn new(projection: Projection<R, P, A>, store: Store<R>) -> Self {
        Self { projection, store }
    }

    pub fn dispatch(&self, action: A) {
        self.store.dispatch((self.projection.action_mapper)(action))
    }
}

impl<R, P, A> StoreHandle for UseProjectionHandle<R, P, A>
where
    R: Reducible,
{
    type Action = A;

    fn dispatch(&self, action: A) {
        UseProjectionHandle::dispatch(self, action)
    }
}

impl<R, P, A> Deref for UseProjectionHandle<R, P, A>
where
    R: Reducible,
{
    type Target = P;

    fn deref(&self) -> &Self::Target {
        (self.projection.state_mapper)(&self.store)
    }
}

impl<R, P, A> Clone for UseProjectionHandle<R, P, A>
where
    R: Reducible,
{
    fn clone(&self) -> Self {
        Self {
            projection: self.projection.clone(),
            store: self.store.clone(),
        }
    }
}

pub fn use_projection<R, P, A>(projection: Projection<R, P, A>) -> UseProjectionHandle<R, P, A>
where
    R: Reducible2,
{
    UseProjectionHandle::new(
        projection,
        use_context::<Store<R>>().expect("No Store context found"),
    )
}

pub struct Store<R>
where
    R: Reducible,
{
    store: UseReducerHandle<R>,
    middleware: Rc<dyn Fn(&dyn StoreHandle<Target = R, Action = R::Action>, R::Action)>,
}

impl<R> Store<R>
where
    R: Reducible,
{
    pub fn new(store: UseReducerHandle<R>) -> Self {
        Self {
            store,
            middleware: Rc::new(|store, action| store.dispatch(action)),
        }
    }

    pub fn apply(
        self,
        middleware: impl Fn(&dyn StoreHandle<Target = R, Action = R::Action>, R::Action) + 'static,
    ) -> Self
    where
        R: 'static,
    {
        Self {
            store: self.store, // TODO XXX FIXME
            middleware: Rc::new(middleware),
        }
    }
}

impl<R> StoreHandle for Store<R>
where
    R: Reducible,
{
    type Action = R::Action;

    fn dispatch(&self, action: Self::Action) {
        (self.middleware)(&self.store, action)
    }
}

impl<R> Deref for Store<R>
where
    R: Reducible,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<R> Clone for Store<R>
where
    R: Reducible,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<R> PartialEq for Store<R>
where
    R: Reducible + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.store == other.store && &*self.middleware as *const _ == &*other.middleware as *const _
    }
}

impl<R> Debug for Store<R>
where
    R: Reducible + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish()
    }
}

pub fn log<R: Reducible>(
    level: Level,
) -> impl Fn(&dyn StoreHandle<Target = R, Action = R::Action>, R::Action)
where
    R: Debug,
    R::Action: Debug,
{
    move |store, action| {
        log!(
            level,
            "BEFORE: Store: {:?}, action: {:?}",
            store.deref(),
            action
        );

        store.dispatch(action);

        log!(level, "AFTER: Store: {:?}", store.deref());
    }
}

pub trait Reducible2: Reducible + Clone + PartialEq + 'static
/*where <Self as Reducible>::Action: PartialEq + 'static*/
{
}

impl<T> Reducible2 for T
where
    T: Reducible + Clone + PartialEq + 'static,
    T::Action: PartialEq + 'static,
{
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ValueState<S>(S);

impl<S> ValueState<S> {
    pub fn new(value: S) -> Self {
        Self(value)
    }
}

impl<S> Reducible for ValueState<S> {
    type Action = ValueAction<S>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let this = match action {
            Self::Action::Update(value) => Self(value),
        };

        this.into()
    }
}

impl<S> Deref for ValueState<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueAction<S> {
    Update(S),
}
