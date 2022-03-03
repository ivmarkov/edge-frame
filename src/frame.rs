use std::fmt::Debug;
use std::vec::*;

use yew::prelude::*;
use yew_router::prelude::*;

use embedded_svc::utils::rest::role::Role;

use crate::plugin::*;
use crate::utils::*;

#[derive(Properties, Clone, PartialEq)]
pub struct FrameProps<R>
where
    R: Routable + PartialEq + Clone,
{
    #[prop_or_default]
    pub app_title: String,

    #[prop_or(Vec::new())]
    pub navigation: Vec<NavigationPlugin<R>>,

    #[prop_or(Vec::new())]
    pub content: Vec<ContentPlugin<R>>,

    // TODO: Most likely should be state
    #[prop_or_default]
    pub active_role: Role,

    pub api_endpoint: Option<APIEndpoint>,
}

impl<R> Default for FrameProps<R>
where
    R: Routable + PartialEq + Clone,
{
    fn default() -> Self {
        FrameProps {
            app_title: "".into(),
            navigation: Vec::new(),
            content: Vec::new(),
            active_role: Role::Admin,
            api_endpoint: None,
        }
    }
}

#[function_component(Frame)]
pub fn frame<R>(props: &FrameProps<R>) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let props = props.clone();

    let open = use_state(|| false);

    html! {
        <BrowserRouter>
            <Switch<R> render={Switch::render(move |routable: &R| view(&props, *open, routable))}/>
        </BrowserRouter>
    }
}

fn view<R>(props: &FrameProps<R>, open: bool, routable: &R) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let routable = routable.clone();

    let normal = get_plugins(props, |nav| {
        nav.insertion_point == InsertionPoint::Navigation && nav.category != Category::Settings
    });

    let settings = get_plugins(props, |nav| {
        nav.insertion_point == InsertionPoint::Navigation && nav.category == Category::Settings
    });

    let status = get_plugins(props, |nav| nav.insertion_point == InsertionPoint::Status);

    if normal.is_empty() && settings.is_empty() {
        return html! {
            <>
                { view_content(props, routable.clone()) }
            </>
        };
    }

    html! {
        <>
        <nav class="navbar" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <a class="navbar-item" href="https://bulma.io">
                    <img src="https://bulma.io/images/bulma-logo.png" width="112" height="28"/>
                </a>

                <a href="#" role="button" class={classes!("navbar-burger", if_true(open, "is-active"))} aria-label="menu" aria-expanded="false" data-target="navbarBasicExample">
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div id="navbarBasicExample" class={classes!("navbar-menu", if_true(open, "is-active"))}>
                <div class="navbar-start">
                    { view_plugins(props, None, &normal, routable) }

                    {
                        if !settings.is_empty() {
                            html! {
                                <div class="navbar-item has-dropdown is-hoverable">
                                    <a href="#" class="navbar-link">{"Settings"}</a>

                                    <div class="navbar-dropdown">
                                        { view_plugins(props, None, &settings, routable) }
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>

                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class="buttons">
                            { view_plugins(props, None, &status, routable) }
                        </div>
                    </div>
                </div>
            </div>
        </nav>
        { view_content(props, routable.clone()) }
        </>
    }
}

fn view_content<R>(props: &FrameProps<R>, routable: R) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let plugins = props
        .content
        .iter()
        .map(|cnt| cnt.component.clone())
        .collect::<Vec<_>>();

    view_plugins(
        props, None, //Some(get_app_bar_renderer(props, routable)),
        &plugins, routable,
    )
}

fn view_plugins<R>(
    props: &FrameProps<R>,
    app_bar_renderer: Option<Callback2<(), Html>>,
    plugins: &[Callback2<PluginProps<R>, Html>],
    routable: R,
) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    html! {
        for plugins.iter().map(|component|
            component.call(PluginProps {
                active_route: routable,
                active_role: props.active_role,
                api_endpoint: props.api_endpoint.clone(),
                app_bar_renderer: app_bar_renderer.clone(),
            }))
    }
}

fn get_plugins<R, F: Fn(&NavigationPlugin<R>) -> bool>(
    props: &FrameProps<R>,
    criteria: F,
) -> Vec<Callback2<PluginProps<R>, Html>>
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    props
        .navigation
        .iter()
        .filter(|nav| criteria(&nav))
        .map(|nav| nav.component.clone())
        .collect()
}
