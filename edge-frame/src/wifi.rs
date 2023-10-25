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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum EditScope {
    STA,
    AP,
    Mixed,
}

impl Default for EditScope {
    fn default() -> Self {
        Self::STA
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
    pub edit_scope: EditScope, // TODO
}

#[function_component(Wifi)]
pub fn wifi(_props: &WifiProps) -> Html {
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

    let router_enabled = use_state(|| initial_router_conf.is_some());
    let client_enabled = use_state(|| initial_client_conf.is_some());

    let new_conf = || {
        let ap_state = ap_state.clone();
        let sta_state = sta_state.clone();
        let router_state = router_state.clone();
        let client_state = client_state.clone();
        let router_enabled = router_enabled.clone();
        let client_enabled = client_enabled.clone();

        move || WifiConfState {
            configuration: Configuration::Mixed(
                sta_state.conf().cloned().unwrap_or(Default::default()),
                ap_state.conf().cloned().unwrap_or(Default::default()),
            ),
            ap_ip_conf: router_enabled
                .then(|| router_state.conf().cloned().unwrap_or(Default::default())),
            sta_ip_conf: client_enabled
                .then(|| client_state.conf().cloned().unwrap_or(Default::default())),
        }
    };

    let onclick = {
        let new_conf = new_conf();

        Callback::from(move |_| {
            dispatch::invoke(new_conf());
        })
    };

    let mobile = true;

    let ap_active = use_state(|| true);
    let switch = {
        let ap_active = ap_active.clone();
        Callback::from(move |_| ap_active.set(!*ap_active))
    };

    let new_conf = new_conf();

    html! {
        <>
        <div class="container">
        {
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
                        {
                            if *ap_active {
                                html! {
                                    <>
                                    <Ap conf={initial_ap_conf.unwrap_or_default()} state_changed={to_callback(ap_state.setter())}/>

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

                                    <Router conf={initial_router_conf.unwrap_or_default()} disabled={!*router_enabled} state_changed={to_callback(router_state.setter())}/>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                    <Sta conf={initial_sta_conf.unwrap_or_default()} state_changed={to_callback(sta_state.setter())}/>

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

                                    <Client conf={initial_client_conf.unwrap_or_default()} disabled={!*client_enabled} state_changed={to_callback(client_state.setter())}/>
                                    </>
                                }
                            }
                        }
                    </div>
                    </>
                }
            } else {
                html! {
                    <div class="tile is-ancestor">
                        <div class="tile is-4 is-vertical is-parent">
                            <div class="tile is-child box">
                                <p class={classes!("title", if_true(matches!(&*ap_state, ApState::Errors), "is-danger"))}>{format!("Access Point{}", if !matches!(&*ap_state, ApState::Unchanged) { "*" } else { "" })}</p>
                                <Ap conf={initial_ap_conf.unwrap_or_default()} state_changed={to_callback(ap_state.setter())}/>

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

                                <Router conf={initial_router_conf.unwrap_or_default()} disabled={!*router_enabled} state_changed={to_callback(router_state.setter())}/>
                            </div>
                        </div>
                        <div class="tile is-4 is-vertical is-parent">
                            <div class="tile is-child box">
                                <p class={classes!("title", if_true(matches!(&*sta_state, StaState::Errors), "is-danger"))}>{format!("Client{}", if !matches!(&*sta_state, StaState::Unchanged) { "*" } else { "" })}</p>
                                <Sta conf={initial_sta_conf.unwrap_or_default()} state_changed={to_callback(sta_state.setter())}/>

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

                                <Client conf={initial_client_conf.unwrap_or_default()} disabled={!*client_enabled} state_changed={to_callback(client_state.setter())}/>
                            </div>
                        </div>
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
