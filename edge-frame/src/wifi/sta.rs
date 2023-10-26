use strum::*;

use yew::prelude::*;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration};

use crate::field::*;
use crate::util::*;

pub type StaConf = ClientConfiguration;

#[derive(Clone)]
pub enum StaState {
    Unchanged,
    Errors,
    Conf(StaConf),
}

impl StaState {
    pub fn conf(&self) -> Option<&StaConf> {
        if let Self::Conf(conf) = self {
            Some(conf)
        } else {
            None
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct StaProps {
    #[prop_or_default]
    pub conf: StaConf,

    #[prop_or_default]
    pub disabled: bool,

    pub state_changed: Callback<StaState>,
}

#[function_component(Sta)]
pub fn sta(props: &StaProps) -> Html {
    let conf = &props.conf;
    let disabled = props.disabled;

    let ssid = Field::text(conf.ssid.as_str().to_owned(), use_state(|| None), Ok);
    let auth = Field::text(
        conf.auth_method.to_string(),
        use_state(|| None),
        |raw_value| {
            Ok(AuthMethod::iter()
                .find(|auth| auth.to_string() == raw_value)
                .unwrap_or_default())
        },
    );
    let password = Field::text(
        conf.password.as_str().to_owned(),
        use_state(|| None),
        |password| {
            if password.is_empty() {
                Err("Password cannot be empty".into())
            } else {
                Ok(password)
            }
        },
    );
    let password_confirm = {
        let password = password.clone();

        Field::text(
            conf.password.as_str().to_owned(),
            use_state(|| None),
            move |raw_text| {
                if raw_text == password.raw_value() {
                    Ok(raw_text)
                } else {
                    Err("Passwords do not match".into())
                }
            },
        )
    };

    let state_changed = props.state_changed.clone();

    let update_state = {
        let ssid = ssid.clone();
        let auth = auth.clone();
        let password = password.clone();
        let password_confirm = password_confirm.clone();

        move |()| {
            let has_errors = !disabled
                && (ssid.has_errors()
                    || auth.has_errors()
                    || auth.value() != Some(AuthMethod::None)
                        && (password.has_errors() || password_confirm.has_errors()));

            let state = if has_errors {
                StaState::Errors
            } else {
                let is_dirty = ssid.is_dirty()
                    || auth.is_dirty()
                    || auth.value() != Some(AuthMethod::None)
                        && (password.is_dirty() || password_confirm.is_dirty());

                if !is_dirty {
                    StaState::Unchanged
                } else {
                    StaState::Conf(ClientConfiguration {
                        ssid: ssid.value().unwrap().as_str().into(),

                        auth_method: auth.value().unwrap(),
                        password: password.value().unwrap_or_default().as_str().into(),
                        ..Default::default()
                    })
                }
            };

            state_changed.emit(state);
        }
    };

    let update_state = Callback::from(update_state);

    let hidden = if_true(disabled, "visibility: hidden;");
    let input_class = |errors| classes!("input", if_true(!disabled && errors, "is-danger"));

    html! {
        <>
        // SSID
        <div class="field">
            <label class="label">{ "SSID" }</label>
            <div class="control">
                <input
                    class={input_class(ssid.has_errors())}
                    type="text"
                    placeholder="0..24 characters"
                    value={ssid.raw_value()}
                    {disabled}
                    oninput={ssid.change(update_state.clone())}
                    />
            </div>
            <p class="help is-danger" style={hidden}>{ssid.error_str()}</p>
        </div>

        // Authentication
        <div class="field">
            <label class="label">{"Authentication"}</label>
            <div class="control">
                <div class="select">
                    <select disabled={disabled} onchange={auth.change(update_state.clone())}>
                    {
                        AuthMethod::iter().map(|item| {
                            html! {
                                <option value={item.to_string()} selected={Some(item) == auth.value()}>
                                    {item.get_message().map(str::to_owned).unwrap_or_else(|| item.to_string())}
                                </option>
                            }
                        })
                        .collect::<Html>()
                    }
                    </select>
                </div>
            </div>
        </div>

        {
            if auth.value() != Some(AuthMethod::None) {
                html! {
                    <>
                    // Password
                    <div class="field">
                        <label class="label">{if auth.value() == Some(AuthMethod::WEP) { "Key" } else { "Password" }}</label>
                        <div class="control">
                            <input
                                class={input_class(password.has_errors())}
                                type="password"
                                placeholder="0..24 characters"
                                value={password.raw_value()}
                                disabled={disabled}
                                oninput={password.change(update_state.clone())}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{password.error_str()}</p>
                    </div>

                    // Confirm password
                    <div class="field">
                        <label class="label">{if auth.value() == Some(AuthMethod::WEP) { "Key Confirmation" } else { "Password Confirmation" }}</label>
                        <div class="control">
                            <input
                                class={input_class(password_confirm.has_errors())}
                                type="password"
                                placeholder="0..24 characters"
                                value={password_confirm.raw_value()}
                                disabled={disabled}
                                oninput={password_confirm.change(update_state.clone())}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{password_confirm.error_str()}</p>
                    </div>
                    </>
                }
            } else {
                html! {}
            }
        }
        </>
    }
}
