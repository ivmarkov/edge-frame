#![recursion_limit = "1024"]

use yew::prelude::*;
use yew_router::prelude::*;

use embedded_svc::edge_config::role::Role;

use edge_frame::components::frame;
use edge_frame::components::wifi;
use edge_frame::components::wifi_ap;

use edge_frame::plugins::ContentPlugin;

#[derive(Debug, Switch, Copy, Clone, PartialEq, Eq, Hash)]
enum Routes {
    #[to = "/ap"]
    AP,
    #[to = "/"]
    Root,
}

#[derive(Properties, Clone, PartialEq, Default)]
pub struct Props {}

pub struct Main;

impl Component for Main {
    type Message = ();
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let wifi = wifi::plugin().map(Routes::Root);
        let wifi_ap = wifi_ap::plugin().map(Routes::AP);

        let nav = wifi.iter().chain(wifi_ap.iter()).collect::<Vec<_>>();
        let content = std::vec!(ContentPlugin::from(&wifi), ContentPlugin::from(&wifi_ap));

        html! {
            <frame::Frame<Routes>
                active_role={Role::Admin}
                api_endpoint={None}
                navigation={nav}
                content={content}
                />
        }
    }
}

pub fn main() {
    App::<Main>::new().mount_to_body();
}
