#![recursion_limit = "1024"]

use std::rc::Rc;

use log::Level;

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
            role: Rc::new(ValueState::new(RoleStateValue::Role(RoleValue::Admin))),
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
        match action {
            AppAction::UpdateRole(action) => Self {
                role: self.role.clone().reduce(action),
                ..(*self).clone()
            },
            AppAction::UpdateWifi(action) => Self {
                wifi: self.wifi.clone().reduce(action),
                ..(*self).clone()
            },
        }
        .into()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    UpdateRole(RoleAction),
    UpdateWifi(WifiAction),
}

#[derive(Debug, Routable, Copy, Clone, PartialEq, Eq, Hash)]
enum Routes {
    #[at("/wifi")]
    Wifi,
    #[at("/authstate")]
    AuthState,
    #[at("/")]
    Home,
}

#[function_component(App)]
fn app() -> Html {
    let store = use_store(|| Rc::new(AppState::new())).apply(log(Level::Info));

    html! {
        <ContextProvider<UseStoreHandle<AppState>> context={store}>
            <BrowserRouter>
                <Switch<Routes> render={Switch::render(render)}/>
            </BrowserRouter>
        </ContextProvider<UseStoreHandle<AppState>>>
    }
}

fn render(route: &Routes) -> Html {
    html! {
        <Frame
            app_title="EDGE FRAME"
            app_url="https://github.com/ivmarkov/edge-frame">
            <Nav>
                <Role<AppState> role={RoleValue::Admin} projection={AppState::role()}>
                    <RouteNavItem<Routes> text="Home" icon="fa-solid fa-house" route={Routes::Home}/>
                </Role<AppState>>
                <Role<AppState> role={RoleValue::Admin} projection={AppState::role()}>
                    <WifiNavItem<Routes> route={Routes::Wifi}/>
                </Role<AppState>>
            </Nav>
            <Status>
                <Role<AppState> role={RoleValue::User} projection={AppState::role()}>
                    <WifiStatusItem<Routes, AppState> route={Routes::Wifi} projection={AppState::wifi()}/>
                </Role<AppState>>
                <RoleLogoutStatusItem<Routes, AppState> auth_status_route={Routes::AuthState} projection={AppState::role()}/>
            </Status>
            <Content>
                {
                    match route {
                        Routes::Home => html! {
                            <Role<AppState> role={RoleValue::User} projection={AppState::role()} auth=true>
                                {"Hello, world!"}
                            </Role<AppState>>
                        },
                        Routes::AuthState => html! {
                            <RoleAuthState<Routes, AppState> home={Some(Routes::Home)} projection={AppState::role()}/>
                        },
                        Routes::Wifi => html! {
                            <Role<AppState> role={RoleValue::Admin} projection={AppState::role()} auth=true>
                                <Wifi<AppState> projection={AppState::wifi()}/>
                            </Role<AppState>>
                        },
                    }
                }
            </Content>
        </Frame>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<App>();
}
