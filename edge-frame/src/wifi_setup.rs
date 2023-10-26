use std::rc::Rc;

use yew::prelude::*;
use yew_router::Routable;
use yewdux_middleware::*;

use crate::frame::{RouteNavItem, RouteStatusItem};
use crate::wifi::{Wifi, WifiConf, WifiConfScope, WifiState};

#[derive(Default, Clone, Debug, Eq, PartialEq, Store)]
pub struct WifiConfStore(pub Option<WifiConf>);

impl Reducer<WifiConfStore> for WifiConf {
    fn apply(self, mut store: Rc<WifiConfStore>) -> Rc<WifiConfStore> {
        let state = Rc::make_mut(&mut store);

        state.0 = Some(self);

        store
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
pub struct WifiSetupProps {
    #[prop_or_default]
    pub conf_scope: WifiConfScope,

    #[prop_or_default]
    pub mobile: bool,
}

#[function_component(WifiSetup)]
pub fn wifi_setup(props: &WifiSetupProps) -> Html {
    let conf_store = use_store_value::<WifiConfStore>();

    let state = use_state(|| WifiState::Unchanged);

    let conf = conf_store.0.as_ref().cloned().unwrap_or(Default::default());

    let state_changed = {
        let state = state.clone();

        Callback::from(move |new_state| state.set(new_state))
    };

    let onclick = {
        let state = state.clone();

        Callback::from(move |_| {
            if let WifiState::Conf(conf) = (&*state).clone() {
                dispatch::invoke(Some(conf));
            }
        })
    };

    html! {
        <div class="container">
        <Wifi conf={conf} conf_scope={props.conf_scope} mobile={props.mobile} state_changed={state_changed}/>

        <input
            type="button"
            class={"button my-4"}
            value="Save"
            disabled={!matches!(&*state, WifiState::Conf(_))}
            {onclick}
        />
        </div>
    }
}
