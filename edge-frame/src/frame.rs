use std::fmt::Debug;

use yew::html::ChildrenRenderer;
use yew::prelude::*;
use yew::virtual_dom::VChild;
use yew_router::prelude::*;

use super::util::*;

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct NavProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Nav)]
pub fn nav(props: &NavProps) -> Html {
    html! {
        <>
            { for props.children.iter() }
        </>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct NavGroupProps {
    /// The text to display.
    #[prop_or_default]
    pub text: String,
    /// The icon to display.
    #[prop_or_default]
    pub icon: Option<String>,

    #[prop_or_default]
    pub children: Children,
}

#[function_component(NavGroup)]
pub fn nav_group(props: &NavGroupProps) -> Html {
    html! {
        <div class="navbar-item has-dropdown is-hoverable">
            <a href="javascript:void(0);" class="navbar-link">
            {
                if let Some(icon) = props.icon.as_ref() {
                    html! {
                        <div style="position:relative">
                            <span class="icon"><i class={icon}></i></span>
                            <span>{props.text.clone()}</span>
                        </div>
                    }
                } else {
                    html! {
                        {props.text.clone()}
                    }
                }
            }
            </a>

            <div class="navbar-dropdown">
                { for props.children.iter() }
            </div>
        </div>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct NavItemProps {
    pub active: bool,

    #[prop_or_default]
    pub selected: Callback<()>,

    /// The text to display.
    #[prop_or_default]
    pub text: String,
    /// The icon to display.
    #[prop_or_default]
    pub icon: Option<String>,
}

#[function_component(NavItem)]
pub fn nav_item(props: &NavItemProps) -> Html {
    let onclick = {
        let selected = props.selected.clone();

        Callback::from(move |_| selected.emit(()))
    };

    html! {
        <a
            class={classes!("navbar-item", if_true(props.active, "is-active"))}
            href="javascript:void(0);"
            {onclick}
        >
        {
            if let Some(icon) = props.icon.as_ref() {
                html! {
                    <div style="position:relative">
                        <span class="icon"><i class={icon}></i></span>
                        <span>{props.text.clone()}</span>
                    </div>
                }
            } else {
                html! {
                    {props.text.clone()}
                }
            }
        }
        </a>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct RouteNavItemProps<R>
where
    R: Routable + Clone,
{
    /// The Switched item representing the route.
    pub route: R,
    /// The text to display.
    #[prop_or_default]
    pub text: String,
    /// The icon to display.
    #[prop_or_default]
    pub icon: Option<String>,
}

#[function_component(RouteNavItem)]
pub fn route_nav_item<R>(props: &RouteNavItemProps<R>) -> Html
where
    R: Routable + Clone + 'static,
{
    let route = use_route::<R>();
    let history = use_history();

    let selected = {
        let route = props.route.clone();

        Callback::from(move |_| {
            let history = history.clone();

            if let Some(history) = history {
                history.push(route.clone())
            }
        })
    };

    html! {
        <NavItem text={props.text.clone()} icon={props.icon.clone()} active={route == Some(props.route.clone())} {selected}/>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct StatusProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Status)]
pub fn status(props: &StatusProps) -> Html {
    html! {
        <>
            { for props.children.iter() }
        </>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct StatusItemProps {
    #[prop_or_default]
    pub selected: Callback<()>,
    /// The icon to display.
    #[prop_or_default]
    pub icon: String,
}

#[function_component(StatusItem)]
pub fn status_item(props: &StatusItemProps) -> Html {
    let onclick = {
        let selected = props.selected.clone();

        Callback::from(move |_| selected.emit(()))
    };

    html! {
        <div class="icon is-large">
            <i class={props.icon.clone()} {onclick}></i>
        </div>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct RouteStatusItemProps<R>
where
    R: Routable + Clone,
{
    /// The Switched item representing the route.
    pub route: R,
    /// The icon to display.
    #[prop_or_default]
    pub icon: String,
}

#[function_component(RouteStatusItem)]
pub fn route_status_item<R>(props: &RouteStatusItemProps<R>) -> Html
where
    R: Routable + Clone + 'static,
{
    let history = use_history();

    let selected = {
        let route = props.route.clone();

        Callback::from(move |_| {
            if let Some(history) = history.as_ref() {
                history.push(route.clone());
            }
        })
    };

    html! {
        <StatusItem icon={props.icon.clone()} {selected}/>
    }
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct ContentProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Content)]
pub fn content(props: &ContentProps) -> Html {
    html! {
        <>
            { for props.children.iter() }
        </>
    }
}

#[derive(Clone, derive_more::From, PartialEq)]
pub enum FrameChild {
    Nav(VChild<Nav>),
    Status(VChild<Status>),
    Content(VChild<Content>),
}

#[allow(clippy::from_over_into)]
impl Into<Html> for FrameChild {
    fn into(self) -> Html {
        match self {
            Self::Nav(child) => child.into(),
            Self::Status(child) => child.into(),
            Self::Content(child) => child.into(),
        }
    }
}

#[derive(Properties, Clone, Default, PartialEq)]
pub struct FrameProps {
    #[prop_or_default]
    pub app_title: String,

    #[prop_or_default]
    pub app_url: String,

    #[prop_or_default]
    pub children: ChildrenRenderer<FrameChild>,
}

#[function_component(Frame)]
pub fn frame(props: &FrameProps) -> Html {
    let props = props.clone();

    let open = use_state(|| false);

    let onclick = {
        let open = open.clone();

        Callback::from(move |_| {
            open.set(!*open);
        })
    };

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

                <div class="navbar-item">
                    <div class="buttons">
                        { for props.children.iter().filter(|child| matches!(child, FrameChild::Status(_))) }
                    </div>
                </div>

                <a
                    href="javascript:void(0);"
                    role="button"
                    class={classes!("navbar-burger", if_true(*open, "is-active"))}
                    aria-label="menu"
                    aria-expanded="false"
                    data-target="navbar"
                    {onclick}
                    >
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div id="navbar" class={classes!("navbar-menu", if_true(*open, "is-active"))}>
                <div class="navbar-start">
                    { for props.children.iter().filter(|child| matches!(child, FrameChild::Nav(_))) }
                </div>

                <div class="navbar-end">
                </div>
            </div>
        </nav>
        { for props.children.iter().filter(|child| matches!(child, FrameChild::Content(_))) }
        </>
    }
}
