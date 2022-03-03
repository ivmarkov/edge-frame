use yew::prelude::*;
use yew_router::prelude::*;

use crate::utils::*;

#[derive(Properties, Clone, Default, Debug, PartialEq)]
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

#[function_component(NavigationItem)]
pub fn navigation_item<R>(props: &Props<R>) -> Html
where
    R: Routable + Clone + 'static,
{
    let history = use_history().unwrap();

    let onclick = {
        let route = props.route.clone();
        Callback::once(move |_| history.push(route))
    };

    html! {
        <a
            class={classes!("navbar-item", if_true(props.active, "is-active"))}
            href="#"
            {onclick}
        >
        {
            if let Some(icon) = props.icon.as_ref() {
                html! {
                    <div style="position:relative">
                        <span class="icon"><i class={icon}></i></span>
                        <span>{props.text.clone()}</span>
                    </div>
                }
            } else {
                html! {
                    {props.text.clone()}
                }
            }
        }
        </a>
    }
}
