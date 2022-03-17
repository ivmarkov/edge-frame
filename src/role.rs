pub use embedded_svc::utils::rest::role::Role as RoleValue;

use yew::prelude::*;

use crate::redust::{use_projection, Projection, Reducible2, ValueAction, ValueState};

pub type RoleAction = ValueAction<RoleValue>;
pub type RoleState = ValueState<RoleValue>;

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RoleProps<R: Reducible2> {
    pub role: RoleValue,
    pub projection: Projection<R, RoleState, RoleAction>,

    #[prop_or_default]
    pub auth: bool,

    #[prop_or_default]
    pub children: Children,
}

#[function_component(Role)]
pub fn role<R: Reducible2>(props: &RoleProps<R>) -> Html {
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
