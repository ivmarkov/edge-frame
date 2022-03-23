pub use embedded_svc::utils::rest::role::Role as RoleValue;

use yew::prelude::*;

use crate::redust::{use_projection, Projection, Reducible2, ValueAction, ValueState};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoleStateValue {
    Unknown,
    Authenticating(Credentials),
    AuthenticationFailed(Credentials),
    Role(embedded_svc::utils::rest::role::Role),
    LoggedOut,
}

impl Default for RoleStateValue {
    fn default() -> Self {
        Self::Unknown
    }
}

pub type RoleAction = ValueAction<RoleStateValue>;
pub type RoleState = ValueState<RoleStateValue>;

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

    match (&**role, &Default::default()) {
        (RoleStateValue::Role(role), _) if *role >= props.role => {
            // Have permissions to render the content
            html! {
                { for props.children.iter() }
            }
        }
        (RoleStateValue::Unknown | RoleStateValue::Authenticating(_), _) if props.auth => {
            // Unknown permissions or authentication in progress => render modal loader if auth=true
            html! {
                <div class="modal is-active">
                    <div class="modal-background"></div>
                    <div class="columns">
                        <div class="column">
                            <div class="loader is-loading"></div>
                        </div>
                    </div>
                </div>
            }
        }
        (RoleStateValue::AuthenticationFailed(credentials), _)
        | (RoleStateValue::Role(RoleValue::None), credentials)
            if props.auth =>
        {
            // Not authenticated yet or previous authentication attempt failed => render login dialog if auth=true
            let submit = {
                let role = role.clone();

                Callback::from(move |credentials| {
                    role.dispatch(RoleAction::Update(RoleStateValue::Authenticating(
                        credentials,
                    )));
                })
            };

            html! {
                <Auth username={credentials.username.clone()} password={credentials.password.clone()} {submit}/>
            }
        }
        _ if props.auth => {
            // No permissions => render permissions denied modal if auth=true
            html! {
                <div class="modal is-active">
                    <div class="modal-background"></div>
                    <div class="modal-content">
                        <div class="box has-text-centered">
                            {"You have no permissions to access this content"}
                        </div>
                    </div>
                </div>
            }
        }
        _ => {
            // In all other cases just hide the content
            // This is when auth=false and we don't have a role (possibly not yet) that allows us to display the content
            html! {}
        }
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct AuthProps {
    #[prop_or_default]
    pub username: String,

    #[prop_or_default]
    pub password: String,

    pub submit: Callback<Credentials>,
}

#[function_component(Auth)]
pub fn auth(props: &AuthProps) -> Html {
    html! {
        <div class="modal is-active">
            <div class="modal-background"></div>
            <div class="modal-content">
                <div class="box has-text-centered">
                </div>
            </div>
        </div>
    }
}
