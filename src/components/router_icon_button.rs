//! A component based on MDC `<ListItem>` that changes the route.

use yew_router::agent::RouteRequest;
use yew_router::prelude::Switch as Routable;
use yew_router::prelude::*;

use yew::prelude::*;
use yew::virtual_dom::VNode;

use material_yew::MatIconButton;

// TODO This should also be PartialEq and Clone. Its blocked on Children not supporting that.
// TODO This should no longer take link & String, and instead take a route: SW implementing Switch
/// Properties for `RouterButton` and `RouterLink`.
#[derive(Properties, Clone, Default, Debug)]
pub struct Props<R>
where
    R: Routable + Clone,
{
    /// The Switched item representing the route.
    pub route: R,
    /// The icon to display.
    #[prop_or_default]
    pub icon: String,
}

/// Message for `RouterButton` and `RouterLink`.
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    /// Tell the router to navigate the application to the Component's pre-defined route.
    Clicked,
}

/// Changes the route when clicked.
#[derive(Debug)]
pub struct RouterIconButton<R: Routable + Clone + 'static, STATE: RouterState = ()> {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<STATE>,
    props: Props<R>,
}

impl<R: Routable + Clone + 'static, STATE: RouterState> Component for RouterIconButton<R, STATE> {
    type Message = Msg;
    type Properties = Props<R>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterIconButton {
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
        html! { <span onclick=cb><MatIconButton icon={self.props.icon.clone()}/></span> }
    }
}
