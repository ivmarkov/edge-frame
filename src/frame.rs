use std::fmt::Debug;
use std::vec::*;

use yew::prelude::*;
use yew_router::prelude::*;

use embedded_svc::utils::rest::role::Role;

use crate::plugin::*;
use crate::utils::*;

#[derive(Properties, Clone, Default, PartialEq)]
pub struct FrameProps<R>
where
    R: Routable + PartialEq + Clone,
{
    #[prop_or_default]
    pub app_title: String,

    #[prop_or_default]
    pub app_url: String,

    #[prop_or(Vec::new())]
    pub navigation: Vec<NavigationPlugin<R>>,

    #[prop_or(Vec::new())]
    pub content: Vec<ContentPlugin<R>>,

    // TODO: Most likely should be state
    #[prop_or(Role::Admin)]
    pub active_role: Role,

    #[prop_or_default]
    pub api_endpoint: Option<APIEndpoint>,
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
            <Switch<R> render={Switch::render(move |routable: &R| render(&props, *open, routable))}/>
        </BrowserRouter>
    }
}

fn render<R>(props: &FrameProps<R>, open: bool, routable: &R) -> Html
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
                { render_content(props, routable.clone()) }
            </>
        };
    }

    html! {
        <>
        <nav class="navbar" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                {
                    if !props.app_title.is_empty() {
                        if props.app_url.is_empty() {
                            html! {
                                <div class="navbar-item is-size-3">{props.app_title.clone()}</div>
                            }
                        } else {
                            html! {
                                <a class="navbar-item is-size-3" href={props.app_url.clone()}>{props.app_title.clone()}</a>
                            }
                        }
                    } else {
                        html! {}
                    }
                }

                <a href="#" role="button" class={classes!("navbar-burger", if_true(open, "is-active"))} aria-label="menu" aria-expanded="false" data-target="navbar">
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div id="navbar" class={classes!("navbar-menu", if_true(open, "is-active"))}>
                <div class="navbar-start">
                    { render_plugins(props, &normal, routable) }

                    {
                        if !settings.is_empty() {
                            html! {
                                <div class="navbar-item has-dropdown is-hoverable">
                                    <a href="#" class="navbar-link">{"Settings"}</a>

                                    <div class="navbar-dropdown">
                                        { render_plugins(props, &settings, routable) }
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
                            { render_plugins(props, &status, routable) }
                        </div>
                    </div>
                </div>
            </div>
        </nav>
        { render_content(props, routable.clone()) }
        </>
    }
}

fn render_content<R>(props: &FrameProps<R>, routable: R) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let plugins = props
        .content
        .iter()
        .map(|cnt| cnt.component.clone())
        .collect::<Vec<_>>();

    render_plugins(props, &plugins, routable)
}

fn render_plugins<R>(
    props: &FrameProps<R>,
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
            }))
    }
}

fn get_plugins<R>(
    props: &FrameProps<R>,
    criteria: impl Fn(&NavigationPlugin<R>) -> bool,
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
