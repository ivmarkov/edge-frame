use yew::prelude::*;

use embedded_svc::ipv4;
use embedded_svc::wifi::Configuration;

use crate::ipv4::client::{Client, ClientState};
use crate::ipv4::router::{Router, RouterState};
use crate::util::*;
use crate::wifi::ap::{Ap, ApState};
use crate::wifi::sta::{Sta, StaState};

pub mod ap;
pub mod sta;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WifiConf {
    pub conf: Configuration,
    pub ap_ip_conf: Option<ipv4::RouterConfiguration>,
    pub sta_ip_conf: Option<ipv4::ClientConfiguration>,
}

impl Default for WifiConf {
    fn default() -> Self {
        Self {
            conf: Configuration::Mixed(Default::default(), Default::default()),
            ap_ip_conf: Some(Default::default()),
            sta_ip_conf: Some(Default::default()),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum WifiIpConfScope {
    Disabled,
    #[default]
    Enabled,
    Optional,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum WifiConfScope {
    Sta(WifiIpConfScope),
    Ap(WifiIpConfScope),
    ApSta(WifiIpConfScope, WifiIpConfScope),
}

impl WifiConfScope {
    pub fn get_sta_ip_conf_scope(&self) -> WifiIpConfScope {
        match self {
            Self::Sta(scope) => *scope,
            Self::ApSta(_, scope) => *scope,
            Self::Ap(_) => WifiIpConfScope::Disabled,
        }
    }

    pub fn get_ap_ip_conf_scope(&self) -> WifiIpConfScope {
        match self {
            Self::Ap(scope) => *scope,
            Self::ApSta(scope, _) => *scope,
            Self::Sta(_) => WifiIpConfScope::Disabled,
        }
    }
}

impl Default for WifiConfScope {
    fn default() -> Self {
        Self::Sta(Default::default())
    }
}

#[derive(Clone)]
pub enum WifiState {
    Unchanged,
    Errors,
    Conf(WifiConf),
}

impl WifiState {
    pub fn conf(&self) -> Option<&WifiConf> {
        if let Self::Conf(conf) = self {
            Some(conf)
        } else {
            None
        }
    }
}

enum Change {
    None,
    Ap(ApState),
    Sta(StaState),
    ApIp(RouterState),
    StaIp(ClientState),
}

impl Change {
    fn ap_state(&self) -> Option<&ApState> {
        if let Self::Ap(state) = self {
            Some(state)
        } else {
            None
        }
    }

    fn sta_state(&self) -> Option<&StaState> {
        if let Self::Sta(state) = self {
            Some(state)
        } else {
            None
        }
    }

    fn ap_ip_state(&self) -> Option<&RouterState> {
        if let Self::ApIp(state) = self {
            Some(state)
        } else {
            None
        }
    }

    fn sta_ip_state(&self) -> Option<&ClientState> {
        if let Self::StaIp(state) = self {
            Some(state)
        } else {
            None
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct WifiProps {
    #[prop_or_default]
    pub conf: WifiConf,

    #[prop_or_default]
    pub conf_scope: WifiConfScope,

    #[prop_or_default]
    pub mobile: bool,

    #[prop_or_default]
    pub disabled: bool,

    pub state_changed: Callback<WifiState, ()>,
}

#[function_component(Wifi)]
pub fn wifi(props: &WifiProps) -> Html {
    let conf = &props.conf;
    let conf_scope = props.conf_scope;
    let disabled = props.disabled;

    let initial_ap_conf = conf.conf.as_ap_conf_ref().cloned();
    let initial_sta_conf = conf.conf.as_client_conf_ref().cloned();
    let initial_router_conf = conf.ap_ip_conf.as_ref().cloned();
    let initial_client_conf = conf.sta_ip_conf.as_ref().cloned();

    let ap_state = use_state(|| ApState::Unchanged);
    let sta_state = use_state(|| StaState::Unchanged);
    let router_state = use_state(|| RouterState::Unchanged);
    let client_state = use_state(|| ClientState::Unchanged);

    let router_enabled = use_state(|| {
        matches!(conf_scope.get_ap_ip_conf_scope(), WifiIpConfScope::Enabled)
            || matches!(conf_scope.get_ap_ip_conf_scope(), WifiIpConfScope::Optional)
                && initial_router_conf.is_some()
    });
    let client_enabled = use_state(|| {
        matches!(conf_scope.get_sta_ip_conf_scope(), WifiIpConfScope::Enabled)
            || matches!(
                conf_scope.get_sta_ip_conf_scope(),
                WifiIpConfScope::Optional
            ) && initial_client_conf.is_some()
    });

    let new_state = || {
        let state_changed = props.state_changed.clone();
        let ap_state = ap_state.clone();
        let sta_state = sta_state.clone();
        let router_state = router_state.clone();
        let client_state = client_state.clone();
        let router_enabled = router_enabled.clone();
        let client_enabled = client_enabled.clone();

        move |change| {
            match &change {
                Change::Ap(state) => ap_state.set(state.clone()),
                Change::Sta(state) => sta_state.set(state.clone()),
                Change::ApIp(state) => router_state.set(state.clone()),
                Change::StaIp(state) => client_state.set(state.clone()),
                _ => (),
            }

            let ap_state = change.ap_state().unwrap_or(&*ap_state);
            let sta_state = change.sta_state().unwrap_or(&*sta_state);
            let router_state = change.ap_ip_state().unwrap_or(&*router_state);
            let client_state = change.sta_ip_state().unwrap_or(&*client_state);
            let router_enabled = router_enabled.clone();
            let client_enabled = client_enabled.clone();

            let state = if matches!(ap_state, ApState::Errors)
                || matches!(sta_state, StaState::Errors)
                || matches!(router_state, RouterState::Errors)
                || matches!(client_state, ClientState::Errors)
            {
                WifiState::Errors
            } else if matches!(ap_state, ApState::Unchanged)
                && matches!(sta_state, StaState::Unchanged)
                && matches!(router_state, RouterState::Unchanged)
                && matches!(client_state, ClientState::Unchanged)
            {
                WifiState::Unchanged
            } else {
                WifiState::Conf(WifiConf {
                    conf: match conf_scope {
                        WifiConfScope::Sta(_) => Configuration::Client(
                            sta_state.conf().cloned().unwrap_or(Default::default()),
                        ),
                        WifiConfScope::Ap(_) => Configuration::AccessPoint(
                            ap_state.conf().cloned().unwrap_or(Default::default()),
                        ),
                        WifiConfScope::ApSta(_, _) => Configuration::Mixed(
                            sta_state.conf().cloned().unwrap_or(Default::default()),
                            ap_state.conf().cloned().unwrap_or(Default::default()),
                        ),
                    },
                    ap_ip_conf: match conf_scope.get_ap_ip_conf_scope() {
                        WifiIpConfScope::Disabled => None,
                        WifiIpConfScope::Enabled => {
                            Some(router_state.conf().cloned().unwrap_or(Default::default()))
                        }
                        WifiIpConfScope::Optional => router_enabled
                            .then(|| router_state.conf().cloned().unwrap_or(Default::default())),
                    },
                    sta_ip_conf: match conf_scope.get_sta_ip_conf_scope() {
                        WifiIpConfScope::Disabled => None,
                        WifiIpConfScope::Enabled => {
                            Some(client_state.conf().cloned().unwrap_or(Default::default()))
                        }
                        WifiIpConfScope::Optional => client_enabled
                            .then(|| client_state.conf().cloned().unwrap_or(Default::default())),
                    },
                })
            };

            state_changed.emit(state);
        }
    };

    let changed_ap = {
        let new_state = new_state();

        Callback::from(move |state| {
            new_state(Change::Ap(state));
        })
    };

    let changed_sta = {
        let new_state = new_state();

        Callback::from(move |state| {
            new_state(Change::Sta(state));
        })
    };

    let changed_ap_ip = {
        let new_state = new_state();

        Callback::from(move |state| {
            new_state(Change::ApIp(state));
        })
    };

    let changed_sta_ip = {
        let new_state = new_state();

        Callback::from(move |state| {
            new_state(Change::StaIp(state));
        })
    };

    let router_changed = || {
        let router_enabled = router_enabled.clone();
        let new_state = new_state();

        Callback::from(move |_| {
            router_enabled.set(!*router_enabled);
            new_state(Change::None);
        })
    };

    let client_changed = || {
        let client_enabled = client_enabled.clone();
        let new_state = new_state();

        Callback::from(move |_| {
            client_enabled.set(!*client_enabled);
            new_state(Change::None);
        })
    };

    let mobile = props.mobile;

    let ap_active = use_state(|| true);
    let switch = {
        let ap_active = ap_active.clone();
        Callback::from(move |_| ap_active.set(!*ap_active))
    };

    let ap_html = || {
        html! {
            <>
            <Ap conf={initial_ap_conf.unwrap_or_default()} state_changed={changed_ap} disabled={disabled}/>

            {
                if matches!(conf_scope.get_ap_ip_conf_scope(), WifiIpConfScope::Optional) {
                    html! {
                        // IP Configuration
                        <div class="field">
                            <label class="checkbox">
                                <input
                                    type="checkbox"
                                    checked={*router_enabled}
                                    onclick={router_changed()}
                                    disabled={disabled}
                                />
                                {"IP Configuration"}
                            </label>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if matches!(conf_scope.get_ap_ip_conf_scope(), WifiIpConfScope::Enabled | WifiIpConfScope::Optional) {
                    html! {
                        <Router conf={initial_router_conf.unwrap_or_default()} disabled={!*router_enabled} state_changed={changed_ap_ip} disabled={disabled}/>
                    }
                } else {
                    html! {}
                }
            }
            </>
        }
    };

    let sta_html = || {
        html! {
            <>
            <Sta conf={initial_sta_conf.unwrap_or_default()} state_changed={changed_sta} disabled={disabled}/>

            {
                if matches!(conf_scope.get_sta_ip_conf_scope(), WifiIpConfScope::Optional) {
                    html! {
                        // IP Configuration
                        <div class="field">
                            <label class="checkbox">
                                <input
                                    type="checkbox"
                                    checked={*client_enabled}
                                    onclick={client_changed()}
                                    disabled={disabled}
                                    />
                                {"IP Configuration"}
                            </label>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if matches!(conf_scope.get_sta_ip_conf_scope(), WifiIpConfScope::Enabled | WifiIpConfScope::Optional) {
                    html! {
                        <Client conf={initial_client_conf.unwrap_or_default()} disabled={!*client_enabled} state_changed={changed_sta_ip} disabled={disabled}/>
                    }
                } else {
                    html! {}
                }
            }
            </>
        }
    };

    html! {
        <>
        <div class="container">
        {
            if matches!(conf_scope, WifiConfScope::ApSta(_, _)) {
                if mobile {
                    html! {
                        <>
                        <div class="tabs">
                            <ul>
                                <li class={if_true(*ap_active, "is-active")}>
                                    <a class={if_true(matches!(&*ap_state, ApState::Errors) || *router_enabled && matches!(&*router_state, RouterState::Errors), "has-text-danger")} href="javascript:void(0);" onclick={switch.clone()}>{format!("Access Point{}", if !matches!(&*ap_state, ApState::Unchanged) { "*" } else { "" })}</a>
                                </li>
                                <li class={if_true(!*ap_active, "is-active")}>
                                    <a class={if_true(matches!(&*sta_state, StaState::Errors) || *client_enabled && matches!(&*client_state, ClientState::Errors), "has-text-danger")} href="javascript:void(0);" onclick={switch}>{format!("Client{}", if !matches!(&*sta_state, StaState::Unchanged) { "*" } else { "" })}</a>
                                </li>
                            </ul>
                        </div>
                        <div>
                            { if *ap_active { ap_html() } else { sta_html() } }
                        </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="tile is-ancestor">
                            <div class="tile is-4 is-vertical is-parent">
                                <div class="tile is-child box">
                                    <p class={classes!("title", if_true(matches!(&*ap_state, ApState::Errors), "is-danger"))}>{format!("Access Point{}", if !matches!(&*ap_state, ApState::Unchanged) { "*" } else { "" })}</p>

                                    { ap_html() }
                                </div>
                            </div>
                            <div class="tile is-4 is-vertical is-parent">
                                <div class="tile is-child box">
                                    <p class={classes!("title", if_true(matches!(&*sta_state, StaState::Errors), "is-danger"))}>{format!("Client{}", if !matches!(&*sta_state, StaState::Unchanged) { "*" } else { "" })}</p>

                                    { sta_html() }
                                </div>
                            </div>
                        </div>
                    }
                }
            } else if matches!(conf_scope, WifiConfScope::Ap(_)) {
                html! {
                    <div class="tile is-child box">
                        { ap_html() }
                    </div>
                }
            } else {
                html! {
                    <div class="tile is-child box">
                        { sta_html() }
                    </div>
                }
            }
        }
        </div>
        </>
    }
}
