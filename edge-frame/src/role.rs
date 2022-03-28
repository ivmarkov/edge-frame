pub use embedded_svc::utils::role::Role as RoleValue;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::auth::*;
use crate::frame::*;
use crate::loading::*;
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
    Role(embedded_svc::utils::role::Role),
    LoggingOut(embedded_svc::utils::role::Role),
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
        (RoleStateValue::Unknown, _) if props.auth => {
            // Unknown permissions => render modal loader if auth=true
            html! {
                <Loading/>
            }
        }
        (RoleStateValue::AuthenticationFailed(credentials), _)
        | (RoleStateValue::Authenticating(credentials), _)
        | (RoleStateValue::Role(RoleValue::None), credentials)
            if props.auth =>
        {
            // Not authenticated yet or previous authentication attempt failed => render login dialog if auth=true
            let submit = {
                let role = role.clone();

                Callback::from(move |(username, password)| {
                    role.dispatch(RoleAction::Update(RoleStateValue::Authenticating(
                        Credentials { username, password },
                    )));
                })
            };

            html! {
                <Auth
                    username={credentials.username.clone()}
                    password={credentials.password.clone()}
                    authenticating={matches!(&**role, RoleStateValue::Authenticating(_))}
                    auth_failed={matches!(&**role, RoleStateValue::AuthenticationFailed(_))}
                    {submit}
                />
            }
        }
        _ if props.auth => {
            // No permissions => render permissions denied modal if auth=true
            html! {
                <NoPerm/>
            }
        }
        _ => {
            // In all other cases just hide the content
            // This is when auth=false and we don't have a role (possibly not yet) that allows us to display the content
            html! {}
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RoleLogoutStatusItemProps<R: Routable + PartialEq + Clone + 'static, S: Reducible2> {
    pub auth_status_route: R,
    pub projection: Projection<S, RoleState, RoleAction>,
}

#[function_component(RoleLogoutStatusItem)]
pub fn role_logout_status_item<R: Routable + PartialEq + Clone + 'static, S: Reducible2>(
    props: &RoleLogoutStatusItemProps<R, S>,
) -> Html {
    let role = use_projection(props.projection.clone());
    let history = use_history();

    match &**role {
        RoleStateValue::Role(role_value)
            if *role_value >= embedded_svc::utils::role::Role::None =>
        {
            let selected = {
                let auth_status_route = props.auth_status_route.clone();
                let role_value = *role_value;
                let role = role.clone();

                Callback::from(move |_| {
                    role.dispatch(RoleAction::Update(RoleStateValue::LoggingOut(role_value)));

                    if let Some(history) = history.as_ref() {
                        history.push(auth_status_route.clone());
                    }
                })
            };

            html! {
                <StatusItem
                    icon="fa-lg fa-solid fa-right-from-bracket"
                    {selected}/>
            }
        }
        _ => {
            html! {}
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RoleAuthStateProps<R: Routable + PartialEq + Clone + 'static, S: Reducible2> {
    #[prop_or_default]
    pub home: Option<R>,
    pub projection: Projection<S, RoleState, RoleAction>,
}

#[function_component(RoleAuthState)]
pub fn role_auth_state<R: Routable + PartialEq + Clone + 'static, S: Reducible2>(
    props: &RoleAuthStateProps<R, S>,
) -> Html {
    let role = use_projection(props.projection.clone());

    let role = match &**role {
        RoleStateValue::Role(role_value) => Some(*role_value),
        _ => None,
    };

    let login = if let Some(home) = props.home.clone() {
        let history = use_history();

        Some(Callback::from(move |_| {
            if let Some(history) = history.as_ref() {
                history.push(home.clone());
            }
        }))
    } else {
        None
    };

    html! {
        <AuthState {login} {role}/>
    }
}
