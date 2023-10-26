#![recursion_limit = "1024"]

use core::fmt::Debug;

use std::rc::Rc;

use edge_frame::wifi::WifiConf;
use log::Level;

use yew::prelude::*;
use yew_router::prelude::*;
use yewdux_middleware::*;

use edge_frame::frame::*;
use edge_frame::middleware::*;
use edge_frame::role::*;
use edge_frame::wifi_setup::*;

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
    use_effect_with((), move |_| {
        init_middleware();

        move || ()
    });

    html! {
        <BrowserRouter>
            <Switch<Routes> render={render}/>
        </BrowserRouter>
    }
}

fn render(route: Routes) -> Html {
    html! {
        <Frame
            app_title="EDGE FRAME"
            app_url="https://github.com/ivmarkov/edge-frame">
            <Nav>
                <Role role={RoleDto::Admin}>
                    <RouteNavItem<Routes> text="Home" icon="fa-solid fa-house" route={Routes::Home}/>
                </Role>
                <Role role={RoleDto::Admin}>
                    <WifiNavItem<Routes> route={Routes::Wifi}/>
                </Role>
            </Nav>
            <Status>
                <Role role={RoleDto::User}>
                    <WifiStatusItem<Routes> route={Routes::Wifi}/>
                </Role>
                <RoleLogoutStatusItem<Routes> auth_status_route={Routes::AuthState}/>
            </Status>
            <Content>
                {
                    match route {
                        Routes::Home => html! {
                            <Role role={RoleDto::User} auth=true>
                                {"Hello, world!"}
                            </Role>
                        },
                        Routes::AuthState => html! {
                            <RoleAuthState<Routes> home={Some(Routes::Home)}/>
                        },
                        Routes::Wifi => html! {
                            <Role role={RoleDto::Admin} auth=true>
                                <WifiSetup/>
                            </Role>
                        },
                    }
                }
            </Content>
        </Frame>
    }
}

fn init_middleware() {
    dispatch::register(store_dispatch::<RoleStore, RoleState>());
    dispatch::register(store_dispatch::<WifiConfStore, WifiConf>());

    dispatch::invoke(RoleState::Role(RoleDto::Admin));
    dispatch::invoke(WifiConf::default());
}

// Set the middleware for each store type
fn store_dispatch<S, M>() -> impl MiddlewareDispatch<M> + Clone
where
    S: Store + Debug,
    M: Reducer<S> + Debug + 'static,
{
    // Update store
    dispatch::store
        // Log store before/after dispatching
        .fuse(Rc::new(log_store(Level::Trace)))
        // Log msg before dispatching
        .fuse(Rc::new(log_msg(Level::Trace)))
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
