use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::*;
use yewdux_middleware::*;

use crate::auth::*;
use crate::frame::*;
use crate::loading::*;

pub use crate::dto::Role as RoleDto;

#[derive(Default, Clone, Debug, PartialEq, Eq, Store)]
pub struct RoleStore(pub Option<RoleState>);

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoleState {
    Authenticating(Credentials),
    AuthenticationFailed(Credentials),
    Role(RoleDto),
    LoggingOut(RoleDto),
    LoggedOut,
}

impl Reducer<RoleStore> for RoleState {
    fn apply(self, mut store: Rc<RoleStore>) -> Rc<RoleStore> {
        let state = Rc::make_mut(&mut store);

        state.0 = Some(self);

        store
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RoleProps {
    pub role: RoleDto,

    #[prop_or_default]
    pub auth: bool,

    #[prop_or_default]
    pub children: Children,
}

#[function_component(Role)]
pub fn role(props: &RoleProps) -> Html {
    let role = use_store_value::<RoleStore>();
    let role = role.0.as_ref();

    match (&role, &Default::default()) {
        (Some(RoleState::Role(role)), _) if *role >= props.role => {
            // Have permissions to render the content
            html! {
                { for props.children.iter() }
            }
        }
        (None, _) if props.auth => {
            // Unknown permissions => render modal loader if auth=true
            html! {
                <Loading/>
            }
        }
        (Some(RoleState::AuthenticationFailed(credentials)), _)
        | (Some(RoleState::Authenticating(credentials)), _)
        | (Some(RoleState::Role(RoleDto::None)), credentials)
            if props.auth =>
        {
            // Not authenticated yet or previous authentication attempt failed => render login dialog if auth=true
            let submit = {
                Callback::from(move |(username, password)| {
                    dispatch::invoke(RoleState::Authenticating(Credentials {
                        username,
                        password,
                    }));
                })
            };

            html! {
                <Auth
                    username={credentials.username.clone()}
                    password={credentials.password.clone()}
                    authenticating={matches!(role, Some(RoleState::Authenticating(_)))}
                    auth_failed={matches!(role, Some(RoleState::AuthenticationFailed(_)))}
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
pub struct RoleLogoutStatusItemProps<R: Routable + PartialEq + Clone + 'static> {
    pub auth_status_route: R,
}

#[function_component(RoleLogoutStatusItem)]
pub fn role_logout_status_item<R: Routable + PartialEq + Clone + 'static>(
    props: &RoleLogoutStatusItemProps<R>,
) -> Html {
    let role = use_store_value::<RoleStore>();
    let role = role.0.as_ref();

    let history = use_navigator();

    match &role {
        Some(RoleState::Role(role_value)) if *role_value >= crate::dto::Role::None => {
            let selected = {
                let auth_status_route = props.auth_status_route.clone();
                let role_value = *role_value;

                Callback::from(move |_| {
                    dispatch::invoke(RoleState::LoggingOut(role_value));

                    if let Some(history) = history.as_ref() {
                        history.push(&auth_status_route);
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
pub struct RoleAuthStateProps<R: Routable + PartialEq + Clone + 'static> {
    #[prop_or_default]
    pub home: Option<R>,
}

#[function_component(RoleAuthState)]
pub fn role_auth_state<R: Routable + PartialEq + Clone + 'static>(
    props: &RoleAuthStateProps<R>,
) -> Html {
    let history = use_navigator();

    let role = use_store_value::<RoleStore>();
    let role = role.0.as_ref();

    let role = match &role {
        Some(RoleState::Role(role_value)) => Some(*role_value),
        _ => None,
    };

    let login = if let Some(home) = props.home.as_ref() {
        let home = home.clone();

        Some(Callback::from(move |_| {
            if let Some(history) = history.as_ref() {
                history.push(&home);
            }
        }))
    } else {
        None
    };

    html! {
        <AuthState {login} {role}/>
    }
}
