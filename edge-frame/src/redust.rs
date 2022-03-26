use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    ops::Deref,
    ptr,
    rc::Rc,
};

use log::{log, Level};

use yew::{use_context, use_mut_ref, use_state, Reducible, UseStateHandle};

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
            &*self.state_mapper as *const _ as *const u8,
            &*other.state_mapper as *const _ as *const u8,
        ) && ptr::eq(
            &*self.action_mapper as *const _ as *const u8,
            &*other.action_mapper as *const _ as *const u8,
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
    store: UseStoreHandle<R>,
}

impl<R, P, A> UseProjectionHandle<R, P, A>
where
    R: Reducible + 'static,
{
    fn new(projection: Projection<R, P, A>, store: UseStoreHandle<R>) -> Self {
        Self { projection, store }
    }

    pub fn dispatch(&self, action: A) {
        self.store.dispatch((self.projection.action_mapper)(action))
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
    R: Reducible + PartialEq + 'static,
{
    UseProjectionHandle::new(
        projection,
        use_context::<UseStoreHandle<R>>().expect("No Store context found"),
    )
}

pub struct StoreProvider<R>(UseStoreHandle<R>)
where
    R: Reducible;

impl<R> StoreProvider<R>
where
    R: Reducible,
{
    pub fn get(&self) -> Ref<'_, Rc<R>> {
        self.0.current.borrow()
    }
}

impl<R> Clone for StoreProvider<R>
where
    R: Reducible,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn use_store<R>(initial_value: impl FnOnce() -> Rc<R> + Clone) -> UseStoreHandle<R>
where
    R: Reducible + 'static,
{
    UseStoreHandle::new(use_mut_ref(initial_value.clone()), use_state(initial_value))
}

pub struct UseStoreHandle<R>
where
    R: Reducible,
{
    current: Rc<RefCell<Rc<R>>>,
    state: UseStateHandle<Rc<R>>,
    dispatcher: Rc<dyn Fn(StoreProvider<R>, R::Action)>,
}

impl<R> UseStoreHandle<R>
where
    R: Reducible + 'static,
{
    fn new(current: Rc<RefCell<Rc<R>>>, state: UseStateHandle<Rc<R>>) -> Self {
        Self {
            current,
            state,
            dispatcher: Rc::new(|store, action| {
                let old_state = store.0.current.borrow().clone();
                let new_state = old_state.reduce(action);

                *store.0.current.borrow_mut() = new_state.clone();
                store.0.state.set(new_state);
            }),
        }
    }

    pub fn dispatch(&self, action: R::Action) {
        let store = (*self).clone();

        (self.dispatcher)(StoreProvider(store), action);
    }

    pub fn apply(
        self,
        middleware: impl Fn(StoreProvider<R>, R::Action, Rc<dyn Fn(StoreProvider<R>, R::Action)>)
            + 'static,
    ) -> Self
    where
        R: 'static,
    {
        let dispatcher = self.dispatcher;

        Self {
            current: self.current.clone(),
            state: self.state.clone(),
            dispatcher: Rc::new(move |store, action| {
                middleware(store, action, dispatcher.clone());
            }),
        }
    }
}

impl<R> Deref for UseStoreHandle<R>
where
    R: Reducible,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.state.deref()
    }
}

impl<R> Clone for UseStoreHandle<R>
where
    R: Reducible,
{
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            state: self.state.clone(),
            dispatcher: self.dispatcher.clone(),
        }
    }
}

impl<R> PartialEq for UseStoreHandle<R>
where
    R: Reducible + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
            && self.state == other.state
            && ptr::eq(
                &*self.dispatcher as *const _ as *const u8,
                &*other.dispatcher as *const _ as *const u8,
            )
    }
}

impl<R> Debug for UseStoreHandle<R>
where
    R: Reducible + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish()
    }
}

pub fn log<R: Reducible>(
    level: Level,
) -> impl Fn(StoreProvider<R>, R::Action, Rc<dyn Fn(StoreProvider<R>, R::Action)>)
where
    R: Clone + Debug,
    R::Action: Debug,
{
    move |store, action, dispatcher| {
        log!(
            level,
            "BEFORE: Store: {:?}, action: {:?}",
            store.get().deref(),
            action
        );

        dispatcher(store.clone(), action);

        log!(level, "AFTER: Store: {:?}", store.get().deref());
    }
}

pub trait Reducible2: Reducible + PartialEq + 'static
/*where <Self as Reducible>::Action: PartialEq + 'static*/
{
}

impl<T> Reducible2 for T
where
    T: Reducible + PartialEq + 'static,
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
