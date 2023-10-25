use std::rc::Rc;

use yew::prelude::*;
use yew_router::Routable;
use yewdux_middleware::*;

use embedded_svc::ipv4;
use embedded_svc::wifi::Configuration;

use crate::frame::{RouteNavItem, RouteStatusItem};
use crate::ipv4::client::{Client, ClientState};
use crate::ipv4::router::{Router, RouterState};
use crate::wifi::ap::{Ap, ApState};
use crate::wifi::sta::{Sta, StaState};
use crate::{to_callback, util::*};

pub mod ap;
pub mod sta;

#[derive(Default, Clone, Debug, Eq, PartialEq, Store)]
pub struct WifiConfStore(pub Option<WifiConfState>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WifiConfState {
    pub configuration: Configuration,
    pub ap_ip_conf: Option<ipv4::RouterConfiguration>,
    pub sta_ip_conf: Option<ipv4::ClientConfiguration>,
}

impl Default for WifiConfState {
    fn default() -> Self {
        Self {
            configuration: Configuration::Mixed(Default::default(), Default::default()),
            ap_ip_conf: Some(Default::default()),
            sta_ip_conf: Some(Default::default()),
        }
    }
}

impl Reducer<WifiConfStore> for WifiConfState {
    fn apply(self, mut store: Rc<WifiConfStore>) -> Rc<WifiConfStore> {
        let state = Rc::make_mut(&mut store);

        state.0 = Some(self);

        store
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum IpConfEditScope {
    Disabled,
    #[default]
    Enabled,
    Optional,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum EditScope {
    Sta(IpConfEditScope),
    Ap(IpConfEditScope),
    ApSta(IpConfEditScope, IpConfEditScope),
}

impl EditScope {
    pub fn get_sta_ip_conf_scope(&self) -> IpConfEditScope {
        match self {
            Self::Sta(scope) => *scope,
            Self::ApSta(_, scope) => *scope,
            Self::Ap(_) => IpConfEditScope::Disabled,
        }
    }

    pub fn get_ap_ip_conf_scope(&self) -> IpConfEditScope {
        match self {
            Self::Ap(scope) => *scope,
            Self::ApSta(scope, _) => *scope,
            Self::Sta(_) => IpConfEditScope::Disabled,
        }
    }
}

impl Default for EditScope {
    fn default() -> Self {
        Self::Sta(Default::default())
    }
}

#[derive(Properties, Clone, Debug, PartialEq, Eq)]
pub struct WifiNavItemProps<R: Routable + PartialEq + Clone + 'static> {
    pub route: R,
}

#[function_component(WifiNavItem)]
pub fn wifi_nav_item<R: Routable + PartialEq + Clone + 'static>(
    props: &WifiNavItemProps<R>,
) -> Html {
    html! {
        <RouteNavItem<R>
            text="Wifi"
            icon="fa-solid fa-wifi"
            route={props.route.clone()}/>
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct WifiStatusItemProps<R: Routable + PartialEq + Clone + 'static> {
    pub route: R,
}

#[function_component(WifiStatusItem)]
pub fn wifi_status_item<R: Routable + PartialEq + Clone + 'static>(
    props: &WifiStatusItemProps<R>,
) -> Html {
    html! {
        <RouteStatusItem<R>
            icon="fa-lg fa-solid fa-wifi"
            route={props.route.clone()}/>
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct WifiProps {
    #[prop_or_default]
    pub edit_scope: EditScope,

    #[prop_or_default]
    pub mobile: bool,
}

#[function_component(Wifi)]
pub fn wifi(props: &WifiProps) -> Html {
    let edit_scope = props.edit_scope;

    let conf_store = use_store_value::<WifiConfStore>();
    let conf = conf_store.0.as_ref();

    let initial_ap_conf = conf.and_then(|c| c.configuration.as_ap_conf_ref().cloned());
    let initial_sta_conf = conf.and_then(|c| c.configuration.as_client_conf_ref().cloned());
    let initial_router_conf = conf.and_then(|c| c.ap_ip_conf.as_ref().cloned());
    let initial_client_conf = conf.and_then(|c| c.sta_ip_conf.as_ref().cloned());

    let ap_state = use_state(|| ApState::Unchanged);
    let sta_state = use_state(|| StaState::Unchanged);
    let router_state = use_state(|| RouterState::Unchanged);
    let client_state = use_state(|| ClientState::Unchanged);

    let router_enabled = use_state(|| {
        matches!(edit_scope.get_ap_ip_conf_scope(), IpConfEditScope::Enabled)
            || matches!(edit_scope.get_ap_ip_conf_scope(), IpConfEditScope::Optional)
                && initial_router_conf.is_some()
    });
    let client_enabled = use_state(|| {
        matches!(edit_scope.get_sta_ip_conf_scope(), IpConfEditScope::Enabled)
            || matches!(
                edit_scope.get_sta_ip_conf_scope(),
                IpConfEditScope::Optional
            ) && initial_client_conf.is_some()
    });

    let new_conf = || {
        let ap_state = ap_state.clone();
        let sta_state = sta_state.clone();
        let router_state = router_state.clone();
        let client_state = client_state.clone();
        let router_enabled = router_enabled.clone();
        let client_enabled = client_enabled.clone();

        move || WifiConfState {
            configuration: match edit_scope {
                EditScope::Sta(_) => {
                    Configuration::Client(sta_state.conf().cloned().unwrap_or(Default::default()))
                }
                EditScope::Ap(_) => Configuration::AccessPoint(
                    ap_state.conf().cloned().unwrap_or(Default::default()),
                ),
                EditScope::ApSta(_, _) => Configuration::Mixed(
                    sta_state.conf().cloned().unwrap_or(Default::default()),
                    ap_state.conf().cloned().unwrap_or(Default::default()),
                ),
            },
            ap_ip_conf: match edit_scope.get_ap_ip_conf_scope() {
                IpConfEditScope::Disabled => None,
                IpConfEditScope::Enabled => {
                    Some(router_state.conf().cloned().unwrap_or(Default::default()))
                }
                IpConfEditScope::Optional => router_enabled
                    .then(|| router_state.conf().cloned().unwrap_or(Default::default())),
            },
            sta_ip_conf: match edit_scope.get_sta_ip_conf_scope() {
                IpConfEditScope::Disabled => None,
                IpConfEditScope::Enabled => {
                    Some(client_state.conf().cloned().unwrap_or(Default::default()))
                }
                IpConfEditScope::Optional => client_enabled
                    .then(|| client_state.conf().cloned().unwrap_or(Default::default())),
            },
        }
    };

    let onclick = {
        let new_conf = new_conf();

        Callback::from(move |_| {
            dispatch::invoke(new_conf());
        })
    };

    let mobile = props.mobile;

    let ap_active = use_state(|| true);
    let switch = {
        let ap_active = ap_active.clone();
        Callback::from(move |_| ap_active.set(!*ap_active))
    };

    let new_conf = new_conf();

    let ap_html = || {
        html! {
            <>
            <Ap conf={initial_ap_conf.unwrap_or_default()} state_changed={to_callback(ap_state.setter())}/>

            {
                if matches!(edit_scope.get_ap_ip_conf_scope(), IpConfEditScope::Optional) {
                    html! {
                        // IP Configuration
                        <div class="field">
                            <label class="checkbox">
                                <input
                                    type="checkbox"
                                    checked={*router_enabled}
                                    onclick={let router_enabled = router_enabled.clone(); Callback::from(move |_| { router_enabled.set(!*router_enabled)})}
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
                if matches!(edit_scope.get_ap_ip_conf_scope(), IpConfEditScope::Enabled | IpConfEditScope::Optional) {
                    html! {
                        <Router conf={initial_router_conf.unwrap_or_default()} disabled={!*router_enabled} state_changed={to_callback(router_state.setter())}/>
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
            <Sta conf={initial_sta_conf.unwrap_or_default()} state_changed={to_callback(sta_state.setter())}/>

            {
                if matches!(edit_scope.get_sta_ip_conf_scope(), IpConfEditScope::Optional) {
                    html! {
                        // IP Configuration
                        <div class="field">
                            <label class="checkbox">
                                <input
                                    type="checkbox"
                                    checked={*client_enabled}
                                    onclick={let client_enabled = client_enabled.clone(); Callback::from(move |_| { client_enabled.set(!*client_enabled)})}
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
                if matches!(edit_scope.get_sta_ip_conf_scope(), IpConfEditScope::Enabled | IpConfEditScope::Optional) {
                    html! {
                        <Client conf={initial_client_conf.unwrap_or_default()} disabled={!*client_enabled} state_changed={to_callback(client_state.setter())}/>
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
            if matches!(edit_scope, EditScope::ApSta(_, _)) {
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
            } else if matches!(edit_scope, EditScope::Ap(_)) {
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

        <input
            type="button"
            class={"button my-4"}
            value="Save"
            disabled={
                matches!(&*ap_state, ApState::Errors)
                || matches!(&*sta_state, StaState::Errors)
                || *router_enabled && matches!(&*router_state, RouterState::Errors)
                || *client_enabled && matches!(&*client_state, ClientState::Errors)
                || conf.cloned().unwrap_or(Default::default()) == new_conf()
            }
            {onclick}
        />
        </div>
        </>
    }
}
