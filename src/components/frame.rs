use std::vec;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::prelude::Switch as RouterSwitch;

use yew_mdc::components::*;
use yew_mdc::components::top_app_bar::*;
use yew_mdc::components::drawer::*;

use crate::lambda;
use crate::plugins::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props<SW: RouterSwitch + Clone> {
    #[prop_or(vec::Vec::new())]
    pub navigation: vec::Vec<NavigationPlugin<SW>>,

    #[prop_or(vec::Vec::new())]
    pub content: vec::Vec<ContentPlugin<SW>>,

    // TODO: Most likely should be state
    #[prop_or_default]
    pub active_role: Role,

    pub api_endpoint: Option<APIEndpoint>,
}

impl<SW: RouterSwitch + Clone> std::default::Default for Props<SW> {
    fn default() -> Self {
        Props {
            navigation: vec::Vec::new(),
            content: vec::Vec::new(),
            active_role: Role::Admin,
            api_endpoint: None,
        }
    }
}

pub struct Frame<SW: 'static + RouterSwitch + Clone + Copy + PartialEq> {
    link: ComponentLink<Self>,
    drawer_open: bool,
    props: Props<SW>,
}

pub enum Msg {
    ToggleDrawer,
}

impl<SW: 'static + RouterSwitch + Clone + Copy + PartialEq> Frame<SW> {
    fn view(props: &Props<SW>, drawer_open: bool, link: &ComponentLink<Self>, routable: Option<SW>) -> Html {
        html! {
            <>
                { Self::view_drawer(props, drawer_open, routable.clone()) }
                { Self::view_app_bar(props, link, routable.clone()) }
                { Self::view_content(props, routable.clone()) }
            </>
        }
    }

    fn view_app_bar(props: &Props<SW>, link: &ComponentLink<Self>, routable: Option<SW>) -> Html {
        let plugins = Self::get_nav_plugins(
            props,
            |nav| nav.insertion_point == InsertionPoint::Appbar);

        let has_drawer_plugins = props.navigation
            .iter()
            .any(|nav| nav.insertion_point == InsertionPoint::Drawer);

        html! {
            <TopAppBar manualrows={true} fixed={true}>
                <div class="mdc-top-app-bar__row">
                    <TopAppBarSection align={section::Align::Start}>
                        {
                            if has_drawer_plugins {
                                html! { <IconButton onclick=link.callback(|_| Msg::ToggleDrawer) classes="material-icons">{"menu"}</IconButton> }
                            } else {
                                html! {}
                            }
                        }
                        <span class="mdc-top-app-bar__title">{"WM1 (SHELLY WATER METER)"}</span>
                    </TopAppBarSection>
                    <TopAppBarSection align={section::Align::End}>
                        <span class="mdc-typography--body2">
                            {"Sat 16:11"}
                        </span>

                        { Self::view_plugins(props, plugins, routable) }

                        <IconButton classes="material-icons">{"power_settings_new"}</IconButton>
                    </TopAppBarSection>
                </div>
            </TopAppBar>
        }
    }

    fn view_drawer(props: &Props<SW>, drawer_open: bool, routable: Option<SW>) -> Html {
        let normal = Self::get_nav_plugins(
            props,
            |nav| nav.insertion_point == InsertionPoint::Drawer && nav.category != Category::Settings);

        let settings = Self::get_nav_plugins(
            props,
            |nav| nav.insertion_point == InsertionPoint::Drawer && nav.category == Category::Settings);

        if normal.is_empty() && settings.is_empty() {
            return html! {}
        }

        html! {
            <Drawer style={Style::Modal} open={drawer_open}>
                <DrawerHeader>
                    <DrawerTitle>{"WATER METER"}</DrawerTitle>
                    <DrawerSubtitle>{"[Admin]"}</DrawerSubtitle>
                    <p/>
                </DrawerHeader>
                <DrawerContent>
                    { Self::view_drawer_plugins(props, Option::None, normal, routable) }
                    { Self::view_drawer_plugins(props, Option::Some("Settings"), settings, routable) }
                </DrawerContent>
            </Drawer>
        }
    }

    fn view_content(props: &Props<SW>, routable: Option<SW>) -> Html {
        let plugins =
            props.content
                .iter()
                .map(|cnt| cnt.component.clone())
                .collect();

        html! {
            <div style={"height: 100%; padding: 1rem;"}>
                { Self::view_plugins(props, plugins, routable) }
            </div>
        }
    }

    fn view_drawer_plugins(props: &Props<SW>, title: Option<&str>, plugins: vec::Vec<lambda::Lambda<PluginProps<SW>, Html>>, routable: Option<SW>) -> Html {
        if !plugins.is_empty() {
            let list = html! {
                <List role={list::Role::ListBox}>
                { Self::view_plugins(props, plugins, routable) }
                </List>
            };

            if let Some(title_str) = title {
                html! {
                    <ListGroup sub_header={title_str}>
                        <ListDivider/>
                        { list }
                    </ListGroup>
                }
            } else {
                list
            }
        } else {
            html! {}
        }
    }

    fn view_plugins(props: &Props<SW>, plugins: vec::Vec<lambda::Lambda<PluginProps<SW>, Html>>, routable: Option<SW>) -> Html {
        html! {
            for plugins.iter().map(|component|
                component.call(PluginProps {
                    active_route: routable,
                    active_role: props.active_role,
                    api_endpoint: props.api_endpoint.clone(),
                }))
        }
    }

    fn get_nav_plugins<F: Fn(&NavigationPlugin<SW>) -> bool>(props: &Props<SW>, criteria: F) -> vec::Vec<lambda::Lambda<PluginProps<SW>, Html>> {
        props.navigation
            .iter()
            .filter(|nav| criteria(&nav))
            .map(|nav| nav.component.clone())
            .collect()
    }
}

impl<SW: 'static + RouterSwitch + Clone + Copy + PartialEq> Component for Frame<SW> {
    type Message = Msg;
    type Properties = Props<SW>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            drawer_open: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleDrawer => {self.drawer_open = !self.drawer_open},
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
            <Router<SW, ()>
                render = Router::render(move |routable: SW|
                    Self::view(&props, drawer_open, &link, Some(routable.clone())))
            />
        }
    }
}
