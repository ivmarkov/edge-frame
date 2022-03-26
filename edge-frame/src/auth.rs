use yew::prelude::*;

use embedded_svc::utils::role::Role;

use crate::field::Field;
use crate::loading::*;
use crate::util::if_true;

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct AuthProps {
    #[prop_or_default]
    pub username: String,

    #[prop_or_default]
    pub password: String,

    #[prop_or_default]
    pub auth_failed: bool,

    #[prop_or_default]
    pub authenticating: bool,

    pub submit: Callback<(String, String)>,
}

#[function_component(Auth)]
pub fn auth(props: &AuthProps) -> Html {
    let mut username = Field::text(Ok);
    let mut password = Field::text(Ok);

    username.update(props.username.clone());
    password.update(props.password.clone());

    let disabled = props.authenticating;
    let hidden = if_true(
        !props.auth_failed || props.authenticating,
        "visibility: hidden;",
    );

    let onclick = {
        let username = username.clone();
        let password = password.clone();
        let submit = props.submit.clone();

        Callback::from(move |_| {
            submit.emit((
                username.value().unwrap_or_default(),
                password.value().unwrap_or_default(),
            ))
        })
    };

    html! {
        <div class="box has-text-centered">
            <div class="field">
                <label class="label">{"Username"}</label>
                <div class="control">
                    <input
                        class="input"
                        type="text"
                        placeholder="0..24 characters"
                        value={username.raw_value()}
                        oninput={username.change()}
                        {disabled}
                        />
                </div>
            </div>
            <div class="field">
                <label class="label">{"Password"}</label>
                <div class="control">
                    <input
                        class="input"
                        type="password"
                        placeholder="0..24 characters"
                        value={password.raw_value()}
                        oninput={password.change()}
                        {disabled}
                        />
                </div>
            </div>
            <p class="help is-danger" style={hidden}>{"Invalid username or password"}</p>
            <button
                class={classes!("button", "my-4", if_true(props.authenticating, "is-loading"))}
                {disabled}
                {onclick}
            >
                {"Login"}
            </button>
        </div>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct AuthStateProps {
    pub role: Option<Role>,
}

#[function_component(AuthState)]
pub fn auth_state(props: &AuthStateProps) -> Html {
    match props.role {
        Some(Role::None) => {
            html! {
                <div class="box has-text-centered">{"You are logged out."}</div>
            }
        }
        None => {
            html! {
                <Loading/>
            }
        }
        Some(role) => {
            html! {
                <div class="box has-text-centered">{format!("You are logged in as {}.", role)}</div>
            }
        }
    }
}

#[function_component(NoPerm)]
pub fn no_perm() -> Html {
    html! {
        <div class="box has-text-centered">
            {"You have no permissions to access this content"}
        </div>
    }
}
