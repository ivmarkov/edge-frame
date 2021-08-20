use std::borrow::Cow;
use std::fmt::Debug;
use std::vec;

use yew::prelude::*;
use yew_router::prelude::Switch as Routed;
use yew_router::prelude::*;

use material_yew::top_app_bar_fixed::*;
use material_yew::*;

use embedded_svc::edge_config::role::Role;

use crate::lambda;
use crate::plugins::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props<R>
where
    R: Routed + PartialEq + Clone,
{
    #[prop_or_default]
    pub app_title: Cow<'static, str>,

    #[prop_or(vec::Vec::new())]
    pub navigation: vec::Vec<NavigationPlugin<R>>,

    #[prop_or(vec::Vec::new())]
    pub content: vec::Vec<ContentPlugin<R>>,

    // TODO: Most likely should be state
    #[prop_or_default]
    pub active_role: Role,

    pub api_endpoint: Option<APIEndpoint>,
}

impl<R> Default for Props<R>
where
    R: Routed + PartialEq + Clone,
{
    fn default() -> Self {
        Props {
            app_title: "".into(),
            navigation: vec::Vec::new(),
            content: vec::Vec::new(),
            active_role: Role::Admin,
            api_endpoint: None,
        }
    }
}

pub struct Frame<R>
where
    R: Routed + PartialEq + Clone + Copy + Debug + 'static,
{
    link: ComponentLink<Self>,
    drawer_open: bool,
    props: Props<R>,
}

pub enum Msg {
    NavIconClick,
    Opened,
    Closed,
}

impl<R> Frame<R>
where
    R: Routed + PartialEq + Clone + Copy + Debug + 'static,
{
    fn view(props: &Props<R>, drawer_open: bool, link: &ComponentLink<Self>, routed: R) -> Html {
        let normal = Self::get_nav_plugins(props, |nav| {
            nav.insertion_point == InsertionPoint::Drawer && nav.category != Category::Settings
        });

        let settings = Self::get_nav_plugins(props, |nav| {
            nav.insertion_point == InsertionPoint::Drawer && nav.category == Category::Settings
        });

        if normal.is_empty() && settings.is_empty() {
            return html! {
                <>
                    { Self::view_content(props, link, routed.clone()) }
                </>
            };
        }

        html! {
            <MatDrawer
                open={drawer_open}
                drawer_type="modal"
                onopened=link.callback(|_| Msg::Opened)
                onclosed=link.callback(|_| Msg::Closed)
            >
                <div class="drawer-content">
                    <drawer::MatDrawerHeader>
                        <drawer::MatDrawerTitle>{"WATER METER"}</drawer::MatDrawerTitle>
                        <drawer::MatDrawerSubtitle>{"[Admin]"}</drawer::MatDrawerSubtitle>
                    </drawer::MatDrawerHeader>

                    <MatList>
                        { Self::view_drawer_plugins(props, Option::None, normal, routed) }
                        { Self::view_drawer_plugins(props, Option::Some("Settings"), settings, routed) }
                    </MatList>
                </div>

                <drawer::MatDrawerAppContent>
                    <div class="app-content">
                        { Self::view_content(props, link, routed.clone()) }
                    </div>
                </drawer::MatDrawerAppContent>
            </MatDrawer>
        }
    }

    fn view_drawer_plugins(
        props: &Props<R>,
        title: Option<&str>,
        plugins: vec::Vec<lambda::Lambda<PluginProps<R>, Html>>,
        routed: R,
    ) -> Html {
        if !plugins.is_empty() {
            let list = html! {
                <MatList activatable=true>
                    { Self::view_plugins(props, None, plugins, routed) }
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

    fn view_app_bar(props: &Props<R>, link: &ComponentLink<Self>, routed: R) -> Html {
        let plugins =
            Self::get_nav_plugins(props, |nav| nav.insertion_point == InsertionPoint::Appbar);

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
                                <span onclick=link.callback(|_| Msg::NavIconClick)><MatIconButton icon="menu"/></span>
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

                    { Self::view_plugins(props, None, plugins, routed) }

                    <MatIconButton icon="power_settings_new"/>
                </MatTopAppBarActionItems>
            </MatTopAppBarFixed>
        }
    }

    fn view_content(props: &Props<R>, link: &ComponentLink<Self>, routed: R) -> Html {
        let plugins = props
            .content
            .iter()
            .map(|cnt| cnt.component.clone())
            .collect();

        Self::view_plugins(
            props,
            Some(Self::get_app_bar_renderer(props, link, routed)),
            plugins,
            routed,
        )
    }

    fn view_plugins(
        props: &Props<R>,
        app_bar_renderer: Option<lambda::Lambda<(), Html>>,
        plugins: vec::Vec<lambda::Lambda<PluginProps<R>, Html>>,
        routed: R,
    ) -> Html {
        html! {
            for plugins.iter().map(|component|
                component.call(PluginProps {
                    active_route: routed,
                    active_role: props.active_role,
                    api_endpoint: props.api_endpoint.clone(),
                    app_bar_renderer: app_bar_renderer.clone(),
                }))
        }
    }

    fn get_app_bar_renderer(
        props: &Props<R>,
        link: &ComponentLink<Self>,
        routed: R,
    ) -> lambda::Lambda<(), Html> {
        let props = props.clone();
        let link = link.clone();

        lambda::Lambda::from(move |_| Self::view_app_bar(&props, &link, routed))
    }

    fn get_nav_plugins<F: Fn(&NavigationPlugin<R>) -> bool>(
        props: &Props<R>,
        criteria: F,
    ) -> vec::Vec<lambda::Lambda<PluginProps<R>, Html>> {
        props
            .navigation
            .iter()
            .filter(|nav| criteria(&nav))
            .map(|nav| nav.component.clone())
            .collect()
    }
}

impl<R> Component for Frame<R>
where
    R: Routed + PartialEq + Clone + Copy + Debug + 'static,
{
    type Message = Msg;
    type Properties = Props<R>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            drawer_open: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NavIconClick => self.drawer_open = !self.drawer_open,
            Msg::Closed => self.drawer_open = false,
            Msg::Opened => self.drawer_open = true,
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let props = self.props.clone();
        let drawer_open = self.drawer_open;
        let link = self.link.clone();

        html! {
            <Router<R>
                render = Router::render(move |routed: R| Self::view(&props, drawer_open, &link, routed.clone()))
            />
        }
    }
}
