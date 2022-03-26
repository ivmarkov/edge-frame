use yew::prelude::*;

#[function_component(Loading)]
pub fn loading() -> Html {
    html! {
        <div class="columns">
            <div class="column">
                <div class="loader is-loading"></div>
            </div>
        </div>
    }
}
