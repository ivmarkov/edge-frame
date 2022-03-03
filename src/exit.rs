use enumset::EnumSet;

use yew::prelude::*;

use embedded_svc::utils::rest::role::Role;

use crate::plugin::*;
use crate::utils::*;

pub fn plugin() -> SimplePlugin<bool> {
    SimplePlugin {
        name: "Exit".into(),
        description: Some("Exits the app".into()),
        icon: Some("fa-lg fa-solid fa-right-from-bracket".into()),
        min_role: Role::None,
        insertion_points: EnumSet::only(InsertionPoint::Status),
        category: Category::Regular,
        route: true,
        component: Callback2::from(move |_: PluginProps<bool>| {
            html! {
                <div class="container">
                    <p class="is-size-2">{"Successful exit"}</p>
                </div>
            }
        }),
    }
}
