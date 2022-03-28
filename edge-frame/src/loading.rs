use yew::prelude::*;

#[function_component(Loading)]
pub fn loading() -> Html {
    html! {
        <div class="columns is-flex is-vcentered">
            <div class="column is-4">
                <div class="loader is-loading"></div>
            </div>
        </div>
    }
}
