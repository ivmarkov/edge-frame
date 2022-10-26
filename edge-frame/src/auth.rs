use yew::prelude::*;

use crate::field::Field;
use crate::loading::*;
use crate::util::if_true;

use crate::dto::Role;

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
        <div class="columns is-flex is-vcentered">
            <div class="column is-4">
                <div class="box has-text-centered">
                    <h3 class="title is-3">{"Login"}</h3>
                    <div class="field">
                        <label class="label">{"Username"}</label>
                        <div class="control">
                            <input
                                class="input"
                                type="text"
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
            </div>
        </div>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct AuthStateProps {
    pub role: Option<Role>,
    #[prop_or_default]
    pub login: Option<Callback<()>>,
}

#[function_component(AuthState)]
pub fn auth_state(props: &AuthStateProps) -> Html {
    match props.role {
        Some(Role::None) => {
            html! {
                <div class="columns is-flex is-vcentered">
                    <div class="column is-4">
                        <div class="box has-text-centered">
                            <h3 class="title is-3">{"You are logged out."}</h3>
                            {
                                if let Some(login) = &props.login {
                                    let login = login.clone();
                                    let onclick = Callback::from(move |_| login.emit(()));

                                    html! {
                                        <a href="javascript:void(0);" {onclick}>{"Login again"}</a>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    </div>
                </div>
            }
        }
        None => {
            html! {
                <Loading/>
            }
        }
        Some(role) => {
            html! {
                <div class="columns is-flex is-vcentered">
                    <div class="column is-4">
                        <div class="box has-text-centered">
                            <h3 class="title is-3">{format!("You are logged in as {role}.")}</h3>
                        </div>
                    </div>
                </div>
            }
        }
    }
}

#[function_component(NoPerm)]
pub fn no_perm() -> Html {
    html! {
        <div class="columns is-flex is-vcentered">
            <div class="column">
                <div class="box has-text-centered">
                    <h3 class="title is-3">{"You have no permissions to access this content"}</h3>
                </div>
            </div>
        </div>
    }
}
