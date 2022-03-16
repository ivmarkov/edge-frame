#![recursion_limit = "1024"]

use std::rc::Rc;

use edge_frame::redust::Projection;
use yew::prelude::*;
use yew_router::prelude::*;

use edge_frame::frame::*;
use edge_frame::role::*;
use edge_frame::wifi::*;

#[derive(Default, Clone, PartialEq)]
pub struct AppStore {
    pub role: Rc<RoleStore>,
    pub wifi: Rc<WifiStore>,
}

impl AppStore {
    pub fn role() -> Projection<AppStore, RoleStore, RoleAction> {
        Projection::new(|store: &AppStore| &*store.role, AppAction::UpdateRole)
    }

    pub fn wifi() -> Projection<AppStore, WifiStore, WifiAction> {
        Projection::new(|store: &AppStore| &*store.wifi, AppAction::UpdateWifi)
    }
}

impl Reducible for AppStore {
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

#[derive(PartialEq)]
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

    html! {
        <Role<AppStore> role={RoleState::User} projection={AppStore::role()} auth=true>
            <Frame
                app_title="EDGE FRAME"
                app_url="https://github.com/ivmarkov/edge-frame">
                <Nav>
                    <Role<AppStore> role={RoleState::Admin} projection={AppStore::role()}>
                        <NavGroup title="Settings">
                            <WifiNavItem<Routes> route={Routes::Wifi}/>
                        </NavGroup>
                    </Role<AppStore>>
                </Nav>
                <Status>
                    <Role<AppStore> role={RoleState::User} projection={AppStore::role()}>
                        <WifiStatusItem<Routes, AppStore> route={Routes::Wifi} projection={AppStore::wifi()}/>
                    </Role<AppStore>>
                </Status>
                <Content>
                    <BrowserRouter>
                        <Switch<Routes> render={Switch::render(render)}/>
                    </BrowserRouter>
                </Content>
            </Frame>
        </Role<AppStore>>
    }
}

fn render(route: &Routes) -> Html {
    match route {
        Routes::Root => html! {
            {"Hello, world!"}
        },
        Routes::Wifi => html! {
            <Role<AppStore> role={RoleState::Admin} projection={AppStore::role()} auth=true>
                <Wifi<AppStore> projection={AppStore::wifi()}/>
            </Role<AppStore>>
        },
    }
}

fn main() {
    yew::start_app::<App>();
}
