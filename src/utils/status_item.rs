use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, Default, Debug, PartialEq)]
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

#[function_component(StatusItem)]
pub fn status_item<R>(props: &Props<R>) -> Html
where
    R: Routable + Clone + 'static,
{
    let history = use_history().unwrap();

    let onclick = {
        let route = props.route.clone();
        Callback::once(move |_| history.push(route))
    };

    html! {
        <div class="icon is-large">
            <i class={props.icon.clone()} {onclick}></i>
        </div>
    }
}
