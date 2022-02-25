use std::fmt::Debug;
use std::vec::*;

use yew::prelude::*;
use yew_router::prelude::*;

use material_yew::top_app_bar_fixed::*;
use material_yew::*;

use embedded_svc::utils::rest::role::Role;

use crate::lambda;
use crate::plugins::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props<R>
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

impl<R> Default for Props<R>
where
    R: Routable + PartialEq + Clone,
{
    fn default() -> Self {
        Props {
            app_title: "".into(),
            navigation: Vec::new(),
            content: Vec::new(),
            active_role: Role::Admin,
            api_endpoint: None,
        }
    }
}

#[function_component(Frame)]
pub fn frame<R>(props: &Props<R>) -> Html
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let drawer_open = false; // TODO
    let props = props.clone();

    html! {
        <Router<R>
            render = { Router::render(move |routable: R| view(&props, drawer_open, routable.clone())) }
        />
    }
}

fn view<R>(props: &Props<R>, drawer_open: bool, routable: R) -> Html 
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let normal = get_nav_plugins(props, |nav| {
        nav.insertion_point == InsertionPoint::Drawer && nav.category != Category::Settings
    });

    let settings = get_nav_plugins(props, |nav| {
        nav.insertion_point == InsertionPoint::Drawer && nav.category == Category::Settings
    });

    if normal.is_empty() && settings.is_empty() {
        return html! {
            <>
                { view_content(props, routable.clone()) }
            </>
        };
    }

    html! {}

        // <MatDrawer
        //     open={drawer_open}
        //     drawer_type="modal"
        //     onopened=link.callback(|_| Msg::Opened)
        //     onclosed=link.callback(|_| Msg::Closed)
        // >
        //     <div class="drawer-content">
        //         <drawer::MatDrawerHeader>
        //             <drawer::MatDrawerTitle>{"WATER METER"}</drawer::MatDrawerTitle>
        //             <drawer::MatDrawerSubtitle>{"[Admin]"}</drawer::MatDrawerSubtitle>
        //         </drawer::MatDrawerHeader>

        //         <MatList>
        //             { Self::view_drawer_plugins(props, Option::None, normal, routable) }
        //             { Self::view_drawer_plugins(props, Option::Some("Settings"), settings, routable) }
        //         </MatList>
        //     </div>

        //     <drawer::MatDrawerAppContent>
        //         <div class="app-content">
        //             { Self::view_content(props, link, routable.clone()) }
        //         </div>
        //     </drawer::MatDrawerAppContent>
        // </MatDrawer>
}

fn view_drawer_plugins<R>(
    props: &Props<R>,
    title: Option<&str>,
    plugins: Vec<lambda::Lambda<PluginProps<R>, Html>>,
    routable: R,
) -> Html 
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    if !plugins.is_empty() {
        let list = html! {
            <MatList activatable=true>
                { view_plugins(props, None, plugins, routable) }
            </MatList>
        };

        if let Some(title) = title {
            html! {
                <>
                    <drawer::MatDrawerSubtitle>{title}</drawer::MatDrawerSubtitle>
                    { list }
                </>
            }
        } else {
            list
        }
    } else {
        html! {}
    }
}

fn view_app_bar<R>(props: &Props<R>, routable: R) -> Html 
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let plugins =
        get_nav_plugins(props, |nav| nav.insertion_point == InsertionPoint::Appbar);

    let has_drawer_plugins = props
        .navigation
        .iter()
        .any(|nav| nav.insertion_point == InsertionPoint::Drawer);

    html! {
        <MatTopAppBarFixed>
            {
                if has_drawer_plugins {
                    html! {
                        <MatTopAppBarNavigationIcon>
                            <MatIconButton icon="menu"/>
                        </MatTopAppBarNavigationIcon>
                    }
                } else {
                    html! {}
                }
            }
            <div slot="title">{"WM1 (SHELLY WATER METER)"}</div>
            <MatTopAppBarActionItems>
                <span class="mdc-typography--body2">
                    {"Sat 16:11"}
                </span>

                { view_plugins(props, None, plugins, routable) }

                <MatIconButton icon="power_settings_new"/>
            </MatTopAppBarActionItems>
        </MatTopAppBarFixed>
    }
}

fn view_content<R>(props: &Props<R>, routable: R) -> Html 
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let plugins = props
        .content
        .iter()
        .map(|cnt| cnt.component.clone())
        .collect();

    view_plugins(
        props,
        Some(get_app_bar_renderer(props, routable)),
        plugins,
        routable,
    )
}

fn view_plugins<R>(
    props: &Props<R>,
    app_bar_renderer: Option<lambda::Lambda<(), Html>>,
    plugins: Vec<lambda::Lambda<PluginProps<R>, Html>>,
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

fn get_app_bar_renderer<R>(
    props: &Props<R>,
    routable: R,
) -> lambda::Lambda<(), Html> 
where
    R: Routable + PartialEq + Clone + Copy + Debug + 'static,
{
    let props = props.clone();

    lambda::Lambda::from(move |_| view_app_bar(&props, routable))
}

fn get_nav_plugins<R, F: Fn(&NavigationPlugin<R>) -> bool>(
    props: &Props<R>,
    criteria: F,
) -> Vec<lambda::Lambda<PluginProps<R>, Html>> 
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
