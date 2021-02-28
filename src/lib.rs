#![recursion_limit="1024"]

use wasm_bindgen::prelude::*;

use yew::prelude::*;
use yew_router::prelude::*;

use components::wifi;
use plugins::Role;
use simple_plugins::SimplePlugin;

pub mod lambda;
pub mod wasm_future;
pub mod components;
pub mod plugins;
pub mod simple_plugins;
pub mod api;

use crate::components::frame;

#[derive(Debug, Switch, Copy, Clone, PartialEq)]
enum Routes {
    #[to = "/"]
    Root,
}

#[derive(Properties, Clone, PartialEq, Default)]
pub struct Props {
}

pub struct Main;

impl Component for Main {
    type Message = ();
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let wifi: simple_plugins::SimplePlugin<Routes> = wifi::plugin().map(
            &|_route| Some(wifi::Routes::Root),
            &|_route| Routes::Root);

        let w2: &SimplePlugin<Routes> = &wifi;

        let nav: std::vec::Vec<plugins::NavigationPlugin<Routes>> = w2.into();

        html! {
            <frame::Frame<Routes>
                active_role={Role::Admin}
                api_endpoint={None}
                navigation={nav},
                content={std::vec!(w2.into())}
                />
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Main>::new().mount_to_body();
}
