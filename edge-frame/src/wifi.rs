use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

use strum::*;

use yew::prelude::*;
use yew_router::Routable;
use yewdux_middleware::*;

use embedded_svc::ipv4::{self, DHCPClientSettings, RouterConfiguration, Subnet};
use embedded_svc::wifi::{
    AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration,
};

use crate::field::*;
use crate::frame::{RouteNavItem, RouteStatusItem};
use crate::util::*;

#[derive(Default, Clone, Debug, Eq, PartialEq, Store)]
pub struct WifiConfStore(pub Option<WifiConfState>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WifiConfState {
    pub configuration: Configuration,
    pub ap_ip_conf: Option<ipv4::RouterConfiguration>,
    pub sta_ip_conf: Option<ipv4::ClientConfiguration>,
}

impl Default for WifiConfState {
    fn default() -> Self {
        Self {
            configuration: Configuration::Mixed(Default::default(), Default::default()),
            ap_ip_conf: Some(Default::default()),
            sta_ip_conf: Some(Default::default()),
        }
    }
}

impl Reducer<WifiConfStore> for WifiConfState {
    fn apply(&self, mut store: Rc<WifiConfStore>) -> Rc<WifiConfStore> {
        let state = Rc::make_mut(&mut store);

        state.0 = Some(self.clone());

        store
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum EditScope {
    STA,
    AP,
    Mixed,
}

impl Default for EditScope {
    fn default() -> Self {
        Self::STA
    }
}

#[derive(Properties, Clone, Debug, PartialEq, Eq)]
pub struct WifiNavItemProps<R: Routable + PartialEq + Clone + 'static> {
    pub route: R,
}

#[function_component(WifiNavItem)]
pub fn wifi_nav_item<R: Routable + PartialEq + Clone + 'static>(
    props: &WifiNavItemProps<R>,
) -> Html {
    html! {
        <RouteNavItem<R>
            text="Wifi"
            icon="fa-solid fa-wifi"
            route={props.route.clone()}/>
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct WifiStatusItemProps<R: Routable + PartialEq + Clone + 'static> {
    pub route: R,
}

#[function_component(WifiStatusItem)]
pub fn wifi_status_item<R: Routable + PartialEq + Clone + 'static>(
    props: &WifiStatusItemProps<R>,
) -> Html {
    html! {
        <RouteStatusItem<R>
            icon="fa-lg fa-solid fa-wifi"
            route={props.route.clone()}/>
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct WifiProps {
    #[prop_or_default]
    pub edit_scope: EditScope, // TODO
}

#[function_component(Wifi)]
pub fn wifi(_props: &WifiProps) -> Html {
    let conf_store = use_store::<WifiConfStore>();
    let conf = conf_store.0.as_ref();

    let mut ap_conf_form = ApConfForm::new();
    let mut sta_conf_form = StaConfForm::new();

    ap_conf_form.update(conf.and_then(|c| {
        c.configuration
            .as_ap_conf_ref()
            .map(|ac| (ac, c.ap_ip_conf.as_ref()))
    }));
    sta_conf_form.update(conf.and_then(|c| {
        c.configuration
            .as_client_conf_ref()
            .map(|cc| (cc, c.sta_ip_conf.as_ref()))
    }));

    let onclick = {
        let sta_conf_form = sta_conf_form.clone();
        let ap_conf_form = ap_conf_form.clone();

        Callback::from(move |_| {
            if let Some((sta_conf, sta_ip_conf)) = sta_conf_form.get() {
                if let Some((ap_conf, ap_ip_conf)) = ap_conf_form.get() {
                    let new_conf = WifiConfState {
                        configuration: Configuration::Mixed(sta_conf, ap_conf),
                        sta_ip_conf,
                        ap_ip_conf,
                    };

                    dispatch::invoke(new_conf);
                }
            }
        })
    };

    let mobile = true;

    let ap_active = use_state(|| true);
    let switch = {
        let ap_active = ap_active.clone();
        Callback::from(move |_| ap_active.set(!*ap_active))
    };

    html! {
        <>
        <div class="container">
        {
            if mobile {
                html! {
                    <>
                    <div class="tabs">
                        <ul>
                            <li class={if_true(*ap_active, "is-active")}>
                                <a class={if_true(ap_conf_form.has_errors(), "has-text-danger")} href="javascript:void(0);" onclick={switch.clone()}>{format!("Access Point{}", if ap_conf_form.is_dirty() { "*" } else { "" })}</a>
                            </li>
                            <li class={if_true(!*ap_active, "is-active")}>
                                <a class={if_true(sta_conf_form.has_errors(), "has-text-danger")} href="javascript:void(0);" onclick={switch}>{format!("Client{}", if sta_conf_form.is_dirty() { "*" } else { "" })}</a>
                            </li>
                        </ul>
                    </div>
                    <div>
                        {
                            if *ap_active {
                                ap_conf_form.render(conf.is_none())
                            } else {
                                sta_conf_form.render(conf.is_none())
                            }
                        }
                    </div>
                    </>
                }
            } else {
                html! {
                    <div class="tile is-ancestor">
                        <div class="tile is-4 is-vertical is-parent">
                            <div class="tile is-child box">
                                <p class={classes!("title", if_true(ap_conf_form.has_errors(), "is-danger"))}>{format!("Access Point{}", if ap_conf_form.is_dirty() { "*" } else { "" })}</p>
                                {ap_conf_form.render(conf.is_none())}
                            </div>
                        </div>
                        <div class="tile is-4 is-vertical is-parent">
                            <div class="tile is-child box">
                                <p class={classes!("title", if_true(sta_conf_form.has_errors(), "is-danger"))}>{format!("Client{}", if sta_conf_form.is_dirty() { "*" } else { "" })}</p>
                                {sta_conf_form.render(conf.is_none())}
                            </div>
                        </div>
                    </div>
                }
            }
        }

        <input
            type="button"
            class={"button my-4"}
            value="Save"
            disabled={
                conf.is_none()
                || ap_conf_form.has_errors()
                || sta_conf_form.has_errors()
                || conf.as_ref().and_then(|conf| conf.configuration.as_ap_conf_ref()) == ap_conf_form.get().as_ref().map(|c| &c.0)
                    && conf.as_ref().and_then(|conf| conf.ap_ip_conf.as_ref()) == ap_conf_form.get().as_ref().and_then(|c| c.1.as_ref())
                    && conf.as_ref().and_then(|conf| conf.configuration.as_client_conf_ref()) == sta_conf_form.get().as_ref().map(|c| &c.0)
                    && conf.as_ref().and_then(|conf| conf.sta_ip_conf.as_ref()) == sta_conf_form.get().as_ref().and_then(|c| c.1.as_ref())
            }
            {onclick}
        />
        </div>
        </>
    }
}

#[derive(Clone)]
struct ApConfForm {
    ssid: TextField<String>,
    hidden_ssid: CheckedField<bool>,

    auth: TextField<AuthMethod>,
    password: TextField<String>,
    password_confirm: TextField<String>,

    ip_conf_enabled: CheckedField<bool>,
    dhcp_server_enabled: CheckedField<bool>,
    subnet: TextField<Subnet>,
    dns: TextField<Option<Ipv4Addr>>,
    secondary_dns: TextField<Option<Ipv4Addr>>,
}

impl ApConfForm {
    fn new() -> Self {
        let password = Field::text(|password| {
            if password.is_empty() {
                Err("Password cannot be empty".into())
            } else {
                Ok(password)
            }
        });

        Self {
            ssid: Field::text(Ok),
            hidden_ssid: Field::checked(Ok),
            auth: Field::text(|raw_value| {
                Ok(AuthMethod::iter()
                    .find(|auth| auth.to_string() == raw_value)
                    .unwrap_or_default())
            }),
            password: password.clone(),
            password_confirm: Field::text(move |raw_text| {
                if raw_text == password.raw_value() {
                    Ok(raw_text)
                } else {
                    Err("Passwords do not match".into())
                }
            }),
            ip_conf_enabled: Field::checked(Ok),
            dhcp_server_enabled: Field::checked(Ok),
            subnet: Field::text(|raw_text| Subnet::from_str(&raw_text).map_err(str::to_owned)),
            dns: TextField::<Option<Ipv4Addr>>::text(|raw_value| {
                if raw_value.trim().is_empty() {
                    Ok(None)
                } else {
                    Ipv4Addr::from_str(&raw_value).map(Some).map_err(|_| {
                        "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned()
                    })
                }
            }),
            secondary_dns: TextField::<Option<Ipv4Addr>>::text(|raw_value| {
                if raw_value.trim().is_empty() {
                    Ok(None)
                } else {
                    Ipv4Addr::from_str(&raw_value).map(Some).map_err(|_| {
                        "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned()
                    })
                }
            }),
        }
    }

    fn has_errors(&self) -> bool {
        self.ssid.has_errors()
            || self.hidden_ssid.has_errors()
            || self.auth.has_errors()
            || self.auth.value() != Some(AuthMethod::None)
                && (self.password.has_errors() || self.password_confirm.has_errors())
            || self.ip_conf_enabled.has_errors()
            || self.ip_conf_enabled.value() == Some(true)
                && (self.dhcp_server_enabled.has_errors()
                    || self.subnet.has_errors()
                    || self.dns.has_errors()
                    || self.secondary_dns.has_errors())
    }

    fn is_dirty(&self) -> bool {
        self.ssid.is_dirty()
            || self.hidden_ssid.is_dirty()
            || self.auth.is_dirty()
            || self.auth.value() != Some(AuthMethod::None)
                && (self.password.is_dirty() || self.password_confirm.is_dirty())
            || self.ip_conf_enabled.is_dirty()
            || self.ip_conf_enabled.value() == Some(true)
                && (self.dhcp_server_enabled.is_dirty()
                    || self.subnet.is_dirty()
                    || self.dns.is_dirty()
                    || self.secondary_dns.is_dirty())
    }

    fn get(&self) -> Option<(AccessPointConfiguration, Option<ipv4::RouterConfiguration>)> {
        if self.has_errors() {
            None
        } else {
            Some((
                AccessPointConfiguration {
                    ssid: self.ssid.value().unwrap().as_str().into(),
                    ssid_hidden: self.hidden_ssid.value().unwrap(),

                    auth_method: self.auth.value().unwrap(),
                    password: self.password.value().unwrap_or_default().as_str().into(),
                    ..Default::default()
                },
                if self.ip_conf_enabled.value().unwrap() {
                    Some(RouterConfiguration {
                        dhcp_enabled: self.dhcp_server_enabled.value().unwrap(),
                        subnet: self.subnet.value().unwrap(),
                        dns: self.dns.value().unwrap(),
                        secondary_dns: self.secondary_dns.value().unwrap(),
                    })
                } else {
                    None
                },
            ))
        }
    }

    fn update(
        &mut self,
        conf: Option<(
            &AccessPointConfiguration,
            Option<&ipv4::RouterConfiguration>,
        )>,
    ) {
        let dconf = (Default::default(), Some(Default::default()));
        let conf = conf.unwrap_or((&dconf.0, dconf.1.as_ref()));

        self.ssid.update(conf.0.ssid.as_str().to_owned());
        self.hidden_ssid.update(conf.0.ssid_hidden);

        self.auth.update(conf.0.auth_method.to_string());
        self.password.update(conf.0.password.as_str().to_owned());
        self.password_confirm
            .update(conf.0.password.as_str().to_owned());

        self.ip_conf_enabled.update(conf.1.is_some());

        self.dhcp_server_enabled
            .update(conf.1.map(|i| i.dhcp_enabled).unwrap_or(false));
        self.subnet
            .update(conf.1.map(|i| i.subnet.to_string()).unwrap_or_default());
        self.dns.update(
            conf.1
                .and_then(|i| i.dns.map(|d| d.to_string()))
                .unwrap_or_default(),
        );
        self.secondary_dns.update(
            conf.1
                .and_then(|i| i.secondary_dns.map(|d| d.to_string()))
                .unwrap_or_default(),
        );
    }

    fn render(&self, disabled: bool) -> Html {
        let disabled_ip = disabled || !self.ip_conf_enabled.value().unwrap_or(false);

        let hidden = if_true(disabled, "visibility: hidden;");
        let hidden_ip = if_true(disabled_ip, "visibility: hidden;");

        let input_class = |errors| classes!("input", if_true(!disabled && errors, "is-danger"));
        let input_class_ip =
            |errors| classes!("input", if_true(!disabled_ip && errors, "is-danger"));

        html! {
            <>
            // SSID
            <div class="field">
                <label class="label">{ "SSID" }</label>
                <div class="control">
                    <input
                        class={input_class(self.ssid.has_errors())}
                        type="text"
                        placeholder="0..24 characters"
                        value={self.ssid.raw_value()}
                        {disabled}
                        oninput={self.ssid.change()}
                        />
                </div>
                <p class="help is-danger" style={hidden}>{self.ssid.error_str()}</p>
            </div>

            // Hide SSID
            <div class="field">
                <label class="checkbox" {disabled}>
                    <input
                        type="checkbox"
                        checked={self.hidden_ssid.raw_value()}
                        {disabled}
                        onclick={self.hidden_ssid.change()}
                    />
                    {"Hidden"}
                </label>
            </div>

            // Authentication
            <div class="field">
                <label class="label">{"Authentication"}</label>
                <div class="control">
                    <div class="select">
                        <select disabled={disabled} onchange={self.auth.change()}>
                        {
                            AuthMethod::iter().map(|item| {
                                html! {
                                    <option value={item.to_string()} selected={Some(item) == self.auth.value()}>
                                        {item.get_message().map(str::to_owned).unwrap_or_else(|| item.to_string())}
                                    </option>
                                }
                            })
                            .collect::<Html>()
                        }
                        </select>
                    </div>
                </div>
            </div>

            {
                if self.auth.value() != Some(AuthMethod::None) {
                    html! {
                        <>
                        // Password
                        <div class="field">
                            <label class="label">{if self.auth.value() == Some(AuthMethod::WEP) { "Key" } else { "Password" }}</label>
                            <div class="control">
                                <input
                                    class={input_class(self.password.has_errors())}
                                    type="password"
                                    placeholder="0..24 characters"
                                    value={self.password.raw_value()}
                                    disabled={disabled}
                                    oninput={self.password.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden}>{self.password.error_str()}</p>
                        </div>

                        // Confirm password
                        <div class="field">
                            <label class="label">{if self.auth.value() == Some(AuthMethod::WEP) { "Key Confirmation" } else { "Password Confirmation" }}</label>
                            <div class="control">
                                <input
                                    class={input_class(self.password_confirm.has_errors())}
                                    type="password"
                                    placeholder="0..24 characters"
                                    value={self.password_confirm.raw_value()}
                                    disabled={disabled}
                                    oninput={self.password_confirm.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden}>{self.password_confirm.error_str()}</p>
                        </div>
                        </>
                    }
                } else {
                    html! {}
                }
            }

            // IP Configuration
            <div class="field">
                <label class="checkbox" {disabled}>
                    <input
                        type="checkbox"
                        checked={self.ip_conf_enabled.raw_value()}
                        {disabled}
                        onclick={self.ip_conf_enabled.change()}
                    />
                    {"IP Configuration"}
                </label>
            </div>

            // DHCP Server
            <div class="field">
                <label class="checkbox" disabled={disabled_ip}>
                    <input
                        type="checkbox"
                        checked={self.dhcp_server_enabled.raw_value()}
                        disabled={disabled_ip}
                        onclick={self.dhcp_server_enabled.change()}
                    />
                    {"DHCP Server"}
                </label>
            </div>

            // Subnet
            <div class="field">
                <label class="label">{ "Subnet" }</label>
                <div class="control">
                    <input
                        class={input_class_ip(self.subnet.has_errors())}
                        type="text"
                        placeholder="XXX.XXX.XXX.XXX/YY"
                        value={self.subnet.raw_value()}
                        disabled={disabled_ip}
                        oninput={self.subnet.change()}
                        />
                </div>
                <p class="help is-danger" style={hidden_ip}>{self.subnet.error_str()}</p>
            </div>

            // DNS
            <div class="field">
                <label class="label">{ "DNS" }</label>
                <div class="control">
                    <input
                        class={input_class_ip(self.dns.has_errors())}
                        type="text"
                        placeholder="XXX.XXX.XXX.XXX (optional)"
                        value={self.dns.raw_value()}
                        disabled={disabled_ip}
                        oninput={self.dns.change()}
                        />
                </div>
                <p class="help is-danger" style={hidden_ip}>{self.dns.error_str()}</p>
            </div>

            // Secondary DNS
            <div class="field">
                <label class="label">{ "Secondary DNS" }</label>
                <div class="control">
                    <input
                        class={input_class_ip(self.secondary_dns.has_errors())}
                        type="text"
                        placeholder="XXX.XXX.XXX.XXX (optional)"
                        value={self.secondary_dns.raw_value()}
                        disabled={disabled_ip}
                        oninput={self.secondary_dns.change()}
                        />
                </div>
                <p class="help is-danger" style={hidden_ip}>{self.secondary_dns.error_str()}</p>
            </div>
            </>
        }
    }
}

#[derive(Clone)]
struct StaConfForm {
    ssid: TextField<String>,

    auth: TextField<AuthMethod>,
    password: TextField<String>,
    password_confirm: TextField<String>,

    ip_conf_enabled: CheckedField<bool>,
    dhcp_enabled: CheckedField<bool>,
    subnet: TextField<Subnet>,
    ip: TextField<Ipv4Addr>,
    dns: TextField<Option<Ipv4Addr>>,
    secondary_dns: TextField<Option<Ipv4Addr>>,
}

impl StaConfForm {
    fn new() -> Self {
        let password = Field::text(|password| {
            if password.is_empty() {
                Err("Password cannot be empty".into())
            } else {
                Ok(password)
            }
        });

        Self {
            ssid: Field::text(Ok),
            auth: Field::text(|raw_value| {
                Ok(AuthMethod::iter()
                    .find(|auth| auth.to_string() == raw_value)
                    .unwrap_or_default())
            }),
            password: password.clone(),
            password_confirm: Field::text(move |raw_text| {
                if raw_text == password.raw_value() {
                    Ok(raw_text)
                } else {
                    Err("Passwords do not match".into())
                }
            }),
            ip_conf_enabled: Field::checked(Ok),
            dhcp_enabled: Field::checked(Ok),
            subnet: Field::text(|raw_text| Subnet::from_str(&raw_text).map_err(str::to_owned)),
            ip: TextField::<Ipv4Addr>::text(|raw_value| {
                Ipv4Addr::from_str(&raw_value)
                    .map_err(|_| "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned())
            }),
            dns: TextField::<Option<Ipv4Addr>>::text(|raw_value| {
                if raw_value.trim().is_empty() {
                    Ok(None)
                } else {
                    Ipv4Addr::from_str(&raw_value).map(Some).map_err(|_| {
                        "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned()
                    })
                }
            }),
            secondary_dns: TextField::<Option<Ipv4Addr>>::text(|raw_value| {
                if raw_value.trim().is_empty() {
                    Ok(None)
                } else {
                    Ipv4Addr::from_str(&raw_value).map(Some).map_err(|_| {
                        "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned()
                    })
                }
            }),
        }
    }

    fn has_errors(&self) -> bool {
        self.ssid.has_errors()
            || self.auth.has_errors()
            || self.auth.value() != Some(AuthMethod::None)
                && (self.password.has_errors() || self.password_confirm.has_errors())
            || self.ip_conf_enabled.has_errors()
            || self.ip_conf_enabled.value() == Some(true)
                && (self.dhcp_enabled.has_errors()
                    || self.dhcp_enabled.value() != Some(true)
                        && (self.subnet.has_errors()
                            || self.ip.has_errors()
                            || self.dns.has_errors()
                            || self.secondary_dns.has_errors()))
    }

    fn is_dirty(&self) -> bool {
        self.ssid.is_dirty()
            || self.auth.is_dirty()
            || self.auth.value() != Some(AuthMethod::None)
                && (self.password.is_dirty() || self.password_confirm.is_dirty())
            || self.ip_conf_enabled.is_dirty()
            || self.ip_conf_enabled.value() == Some(true)
                && (self.dhcp_enabled.is_dirty()
                    || self.dhcp_enabled.value() != Some(true)
                        && (self.subnet.is_dirty()
                            || self.ip.is_dirty()
                            || self.dns.is_dirty()
                            || self.secondary_dns.is_dirty()))
    }

    fn get(&self) -> Option<(ClientConfiguration, Option<ipv4::ClientConfiguration>)> {
        if self.has_errors() {
            None
        } else {
            Some((
                ClientConfiguration {
                    ssid: self.ssid.value().unwrap().as_str().into(),

                    auth_method: self.auth.value().unwrap(),
                    password: self.password.value().unwrap_or_default().as_str().into(),
                    ..Default::default()
                },
                if self.ip_conf_enabled.value().unwrap() {
                    Some(if self.dhcp_enabled.value().unwrap() {
                        ipv4::ClientConfiguration::DHCP(DHCPClientSettings { hostname: None })
                    } else {
                        ipv4::ClientConfiguration::Fixed(ipv4::ClientSettings {
                            subnet: self.subnet.value().unwrap(),
                            ip: self.ip.value().unwrap(),
                            dns: self.dns.value().unwrap(),
                            secondary_dns: self.secondary_dns.value().unwrap(),
                        })
                    })
                } else {
                    None
                },
            ))
        }
    }

    fn update(&mut self, conf: Option<(&ClientConfiguration, Option<&ipv4::ClientConfiguration>)>) {
        let dconf = (Default::default(), Some(Default::default()));
        let conf = conf.unwrap_or((&dconf.0, dconf.1.as_ref()));

        self.ssid.update(conf.0.ssid.as_str().to_owned());

        self.auth.update(conf.0.auth_method.to_string());
        self.password.update(conf.0.password.as_str().to_owned());
        self.password_confirm
            .update(conf.0.password.as_str().to_owned());

        self.ip_conf_enabled.update(conf.1.is_some());

        self.dhcp_enabled.update(
            conf.1
                .map(|i| matches!(i, ipv4::ClientConfiguration::DHCP(_)))
                .unwrap_or(false),
        );
        self.subnet.update(
            conf.1
                .and_then(|i| i.as_fixed_settings_ref().map(|i| i.subnet.to_string()))
                .unwrap_or_default(),
        );
        self.ip.update(
            conf.1
                .and_then(|i| i.as_fixed_settings_ref().map(|i| i.ip.to_string()))
                .unwrap_or_default(),
        );
        self.dns.update(
            conf.1
                .and_then(|i| {
                    i.as_fixed_settings_ref()
                        .and_then(|i| i.dns.map(|d| d.to_string()))
                })
                .unwrap_or_default(),
        );
        self.secondary_dns.update(
            conf.1
                .and_then(|i| {
                    i.as_fixed_settings_ref()
                        .and_then(|i| i.secondary_dns.map(|d| d.to_string()))
                })
                .unwrap_or_default(),
        );
    }

    fn render(&self, disabled: bool) -> Html {
        let disabled_ip = disabled || !self.ip_conf_enabled.value().unwrap_or(false);

        let hidden = if_true(disabled, "visibility: hidden;");
        let hidden_ip = if_true(disabled_ip, "visibility: hidden;");

        let input_class = |errors| classes!("input", if_true(!disabled && errors, "is-danger"));
        let input_class_ip =
            |errors| classes!("input", if_true(!disabled_ip && errors, "is-danger"));

        html! {
            <>
            // SSID
            <div class="field">
                <label class="label">{ "SSID" }</label>
                <div class="control">
                    <input
                        class={input_class(self.ssid.has_errors())}
                        type="text"
                        placeholder="0..24 characters"
                        value={self.ssid.raw_value()}
                        {disabled}
                        oninput={self.ssid.change()}
                        />
                </div>
                <p class="help is-danger" style={hidden}>{self.ssid.error_str()}</p>
            </div>

            // Authentication
            <div class="field">
                <label class="label">{"Authentication"}</label>
                <div class="control">
                    <div class="select">
                        <select disabled={disabled} onchange={self.auth.change()}>
                        {
                            AuthMethod::iter().map(|item| {
                                html! {
                                    <option value={item.to_string()} selected={Some(item) == self.auth.value()}>
                                        {item.get_message().map(str::to_owned).unwrap_or_else(|| item.to_string())}
                                    </option>
                                }
                            })
                            .collect::<Html>()
                        }
                        </select>
                    </div>
                </div>
            </div>

            {
                if self.auth.value() != Some(AuthMethod::None) {
                    html! {
                        <>
                        // Password
                        <div class="field">
                            <label class="label">{if self.auth.value() == Some(AuthMethod::WEP) { "Key" } else { "Password" }}</label>
                            <div class="control">
                                <input
                                    class={input_class(self.password.has_errors())}
                                    type="password"
                                    placeholder="0..24 characters"
                                    value={self.password.raw_value()}
                                    disabled={disabled}
                                    oninput={self.password.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden}>{self.password.error_str()}</p>
                        </div>

                        // Confirm password
                        <div class="field">
                            <label class="label">{if self.auth.value() == Some(AuthMethod::WEP) { "Key Confirmation" } else { "Password Confirmation" }}</label>
                            <div class="control">
                                <input
                                    class={input_class(self.password_confirm.has_errors())}
                                    type="password"
                                    placeholder="0..24 characters"
                                    value={self.password_confirm.raw_value()}
                                    disabled={disabled}
                                    oninput={self.password_confirm.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden}>{self.password_confirm.error_str()}</p>
                        </div>
                        </>
                    }
                } else {
                    html! {}
                }
            }

            // IP Configuration
            <div class="field">
                <label class="checkbox" {disabled}>
                    <input
                        type="checkbox"
                        checked={self.ip_conf_enabled.raw_value()}
                        {disabled}
                        onclick={self.ip_conf_enabled.change()}
                    />
                    {"IP Configuration"}
                </label>
            </div>

            // DHCP
            <div class="field">
                <label class="checkbox" disabled={disabled_ip}>
                    <input
                        type="checkbox"
                        checked={self.dhcp_enabled.raw_value()}
                        disabled={disabled_ip}
                        onclick={self.dhcp_enabled.change()}
                    />
                    {"DHCP"}
                </label>
            </div>

            {
                if !self.dhcp_enabled.value().unwrap_or(true) {
                    html! {
                        <>
                        // Gateway/Subnet
                        <div class="field">
                            <label class="label">{ "Gateway/Subnet" }</label>
                            <div class="control">
                                <input
                                    class={input_class_ip(self.subnet.has_errors())}
                                    type="text"
                                    placeholder="XXX.XXX.XXX.XXX/YY"
                                    value={self.subnet.raw_value()}
                                    disabled={disabled_ip}
                                    oninput={self.subnet.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden_ip}>{self.subnet.error_str()}</p>
                        </div>

                        // IP
                        <div class="field">
                            <label class="label">{ "IP" }</label>
                            <div class="control">
                                <input
                                    class={input_class_ip(self.ip.has_errors())}
                                    type="text"
                                    placeholder="XXX.XXX.XXX.XXX"
                                    value={self.ip.raw_value()}
                                    disabled={disabled_ip}
                                    oninput={self.ip.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden_ip}>{self.ip.error_str()}</p>
                        </div>

                        // DNS
                        <div class="field">
                            <label class="label">{ "DNS" }</label>
                            <div class="control">
                                <input
                                    class={input_class_ip(self.dns.has_errors())}
                                    type="text"
                                    placeholder="XXX.XXX.XXX.XXX (optional)"
                                    value={self.dns.raw_value()}
                                    disabled={disabled_ip}
                                    oninput={self.dns.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden_ip}>{self.dns.error_str()}</p>
                        </div>

                        // Secondary DNS
                        <div class="field">
                            <label class="label">{ "Secondary DNS" }</label>
                            <div class="control">
                                <input
                                    class={input_class_ip(self.secondary_dns.has_errors())}
                                    type="text"
                                    placeholder="XXX.XXX.XXX.XXX (optional)"
                                    value={self.secondary_dns.raw_value()}
                                    disabled={disabled_ip}
                                    oninput={self.secondary_dns.change()}
                                    />
                            </div>
                            <p class="help is-danger" style={hidden_ip}>{self.secondary_dns.error_str()}</p>
                        </div>
                        </>
                    }
                } else {
                    html! {}
                }
            }
            </>
        }
    }
}
