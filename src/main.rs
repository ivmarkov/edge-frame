#![recursion_limit = "1024"]

use yew::prelude::*;
use yew_router::prelude::*;

use embedded_svc::utils::rest::role::Role;

use edge_frame::frame;
use edge_frame::plugin;
use edge_frame::wifi;

#[derive(Debug, Routable, Copy, Clone, PartialEq, Eq, Hash)]
enum Routes {
    #[at("/")]
    Root,
}

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let wifi = wifi::plugin(wifi::PluginBehavior::Mixed).map(Routes::Root);

    let nav = wifi.iter().collect::<Vec<_>>();
    let content = std::vec![plugin::ContentPlugin::from(&wifi)];

    html! {
        <frame::Frame<Routes>
            active_role={Role::Admin}
            api_endpoint={None}
            navigation={nav}
            content={content}
            />
    }

    // html! {
    //     <Wifi1 wifi_endpoint={WifiAsync}/>
    // }
}

fn main() {
    yew::start_app::<App>();
}
