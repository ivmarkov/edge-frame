#![recursion_limit = "1024"]

use std::rc::Rc;

use log::Level;

use log::info;
use yew::prelude::*;
use yew_router::prelude::*;

use edge_frame::frame::*;
use edge_frame::redust::*;
use edge_frame::role::*;
use edge_frame::wifi::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AppState {
    pub role: Rc<RoleState>,
    pub wifi: Rc<WifiState>,
}

impl AppState {
    pub fn new() -> Self {
        //Default::default()
        Self {
            role: Rc::new(ValueState::new(RoleValue::Admin)),
            wifi: Rc::new(ValueState::new(Some(Default::default()))),
        }
    }

    pub fn role() -> Projection<AppState, RoleState, RoleAction> {
        Projection::new(|state: &AppState| &*state.role, AppAction::UpdateRole)
    }

    pub fn wifi() -> Projection<AppState, WifiState, WifiAction> {
        Projection::new(|state: &AppState| &*state.wifi, AppAction::UpdateWifi)
    }
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new = match action {
            AppAction::UpdateRole(action) => Self {
                role: self.role.clone().reduce(action),
                ..(&*self).clone()
            },
            AppAction::UpdateWifi(action) => Self {
                wifi: self.wifi.clone().reduce(action),
                ..(&*self).clone()
            },
        };

        new.into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    UpdateRole(RoleAction),
    UpdateWifi(WifiAction),
}

#[derive(Debug, Routable, Copy, Clone, PartialEq, Eq, Hash)]
enum Routes {
    #[at("/wifi")]
    Wifi,
    #[at("/")]
    Root,
}

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let store = Store::new(use_reducer(|| AppState::new())).apply(log(Level::Info));

    info!("Here!");

    html! {
        <ContextProvider<Store<AppState>> context={store.clone()}>
            <Role<AppState> role={RoleValue::User} projection={AppState::role()} auth=true>
                <Frame
                    app_title="EDGE FRAME"
                    app_url="https://github.com/ivmarkov/edge-frame">
                    <Nav>
                        <Role<AppState> role={RoleValue::Admin} projection={AppState::role()}>
                            <NavGroup title="Settings">
                                <WifiNavItem<Routes> route={Routes::Wifi}/>
                            </NavGroup>
                        </Role<AppState>>
                    </Nav>
                    <Status>
                        <Role<AppState> role={RoleValue::User} projection={AppState::role()}>
                            <WifiStatusItem<Routes, AppState> route={Routes::Wifi} projection={AppState::wifi()}/>
                        </Role<AppState>>
                    </Status>
                    <Content>
                        <BrowserRouter>
                            <Switch<Routes> render={Switch::render(render)}/>
                        </BrowserRouter>
                    </Content>
                </Frame>
            </Role<AppState>>
        </ContextProvider<Store<AppState>>>
    }
}

fn render(route: &Routes) -> Html {
    match route {
        Routes::Root => html! {
            {"Hello, world!"}
        },
        Routes::Wifi => html! {
            <Role<AppState> role={RoleValue::Admin} projection={AppState::role()} auth=true>
                <Wifi<AppState> projection={AppState::wifi()}/>
            </Role<AppState>>
        },
    }
}

fn main() {
    yew::start_app::<App>();
}
