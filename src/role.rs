pub use embedded_svc::utils::rest::role::Role as RoleState;

use yew::prelude::*;

use crate::redust::{use_projection, Projection, SimpleStore, SimpleStoreAction, Store};

pub type RoleAction = SimpleStoreAction<RoleState>;
pub type RoleStore = SimpleStore<RoleState>;

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RoleProps<S: Store> {
    pub role: RoleState,
    pub projection: Projection<S, RoleStore, RoleAction>,

    #[prop_or_default]
    pub auth: bool,

    #[prop_or_default]
    pub children: Children,
}

#[function_component(Role)]
pub fn role<S: Store>(props: &RoleProps<S>) -> Html {
    let role = use_projection(props.projection.clone());

    if **role >= props.role {
        html! {
            { for props.children.iter() }
        }
    } else if props.auth {
        // TODO
        html! {}
    } else {
        html! {}
    }
}
