//! A component based on MDC `<ListItem>` that changes the route.

use yew_router::{
    agent::{RouteAgentDispatcher, RouteRequest},
    route::Route,
    RouterState,
    Switch,
};

use yew::prelude::*;
use yew::virtual_dom::VNode;

use yew_mdc::components::*;
use yew_mdc::components::list::*;

// TODO This should also be PartialEq and Clone. Its blocked on Children not supporting that.
// TODO This should no longer take link & String, and instead take a route: SW implementing Switch
/// Properties for `RouterButton` and `RouterLink`.
#[derive(Properties, Clone, Default, Debug)]
pub struct Props<SW>
where
    SW: Switch + Clone,
{
    /// The Switched item representing the route.
    pub route: SW,
    /// Whether the component represents an active route.
    #[prop_or_default]
    pub active: bool,
    /// The text to display.
    #[prop_or_default]
    pub text: String,
    /// The icon to display.
    #[prop_or_default]
    pub icon: Option<String>,
}

/// Message for `RouterButton` and `RouterLink`.
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    /// Tell the router to navigate the application to the Component's pre-defined route.
    Clicked,
}

/// Changes the route when clicked.
#[derive(Debug)]
pub struct RouterListItem<SW: Switch + Clone + 'static, STATE: RouterState = ()> {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<STATE>,
    props: Props<SW>,
}

impl<SW: Switch + Clone + 'static, STATE: RouterState> Component for RouterListItem<SW, STATE> {
    type Message = Msg;
    type Properties = Props<SW>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterListItem {
            link,
            router,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                let route = Route::from(self.props.route.clone());
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> VNode {
        let cb = self.link.callback(|event: MouseEvent| {
            event.prevent_default();
            Msg::Clicked
        });
        html! {
            <ListItem 
                selected={self.props.active} 
                tabindex={0} 
                role={list_item::Role::Option}
                onclick=cb
            >
                {
                    if let Some(ref icon_str) = self.props.icon {
                        html! { <IconButton classes="material-icons">{icon_str}</IconButton> }
                    } else {
                        html! {}
                    }
                }
                <ListItemText>{self.props.text.clone()}</ListItemText>
            </ListItem>
        }
    }
}
