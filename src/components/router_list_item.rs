//! A component based on MDC `<ListItem>` that changes the route.

use material_yew::list::RequestSelectedDetail;
use yew_router::agent::RouteRequest;
use yew_router::prelude::Switch as Routable;
use yew_router::prelude::*;

use yew::prelude::*;
use yew::virtual_dom::VNode;

use material_yew::list::GraphicType;
use material_yew::*;

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
    None,
}

/// Changes the route when clicked.
#[derive(Debug)]
pub struct RouterListItem<R: Routable + Clone + 'static, STATE: RouterState = ()> {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<STATE>,
    props: Props<R>,
}

impl<R: Routable + Clone + 'static, STATE: RouterState> Component for RouterListItem<R, STATE> {
    type Message = Msg;
    type Properties = Props<R>;

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
            Msg::None => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> VNode {
        let cb = self.link.callback(|event: RequestSelectedDetail| {
            if event.selected {
                Msg::Clicked
            } else {
                Msg::None
            }
        });
        html! {
            <MatListItem
                selected={self.props.active}
                tabindex=0
                activated={self.props.active}
                graphic={if let Some(ref _icon_str) = self.props.icon {GraphicType::Icon} else {GraphicType::Null}}
                on_request_selected=cb
            >
                {
                    if let Some(ref icon_str) = self.props.icon {
                        html! { <span slot="graphic"><MatIcon>{icon_str}</MatIcon></span> }
                    } else {
                        html! {}
                    }
                }
                {self.props.text.clone()}
            </MatListItem>
        }
    }
}
