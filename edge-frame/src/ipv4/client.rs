use std::net::Ipv4Addr;
use std::str::FromStr;

use yew::prelude::*;

use embedded_svc::ipv4::{self, DHCPClientSettings, Subnet};

use crate::field::*;
use crate::util::*;

pub type ClientConf = ipv4::ClientConfiguration;

pub enum ClientState {
    Unchanged,
    Errors,
    Conf(ClientConf),
}

impl ClientState {
    pub fn conf(&self) -> Option<&ClientConf> {
        if let Self::Conf(conf) = self {
            Some(conf)
        } else {
            None
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct ClientProps {
    #[prop_or_default]
    pub conf: ClientConf,

    #[prop_or_default]
    pub disabled: bool,

    pub state_changed: Callback<ClientState>,
}

#[function_component(Client)]
pub fn client(props: &ClientProps) -> Html {
    let conf = &props.conf;
    let disabled = props.disabled;

    let dhcp_enabled = Field::checked(
        matches!(conf, ipv4::ClientConfiguration::DHCP(_)),
        use_state(|| None),
        Ok,
    );
    let subnet = Field::text(
        conf.as_fixed_settings_ref()
            .map(|i| i.subnet.to_string())
            .unwrap_or_default(),
        use_state(|| None),
        |raw_text| Subnet::from_str(&raw_text).map_err(str::to_owned),
    );
    let ip = TextField::<Ipv4Addr>::text(
        conf.as_fixed_settings_ref()
            .map(|i| i.ip.to_string())
            .unwrap_or_default(),
        use_state(|| None),
        |raw_value| {
            Ipv4Addr::from_str(&raw_value)
                .map_err(|_| "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned())
        },
    );
    let dns = TextField::<Option<Ipv4Addr>>::text(
        conf.as_fixed_settings_ref()
            .and_then(|i| i.dns.map(|d| d.to_string()))
            .unwrap_or_default(),
        use_state(|| None),
        |raw_value| {
            if raw_value.trim().is_empty() {
                Ok(None)
            } else {
                Ipv4Addr::from_str(&raw_value)
                    .map(Some)
                    .map_err(|_| "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned())
            }
        },
    );
    let secondary_dns = TextField::<Option<Ipv4Addr>>::text(
        conf.as_fixed_settings_ref()
            .and_then(|i| i.secondary_dns.map(|d| d.to_string()))
            .unwrap_or_default(),
        use_state(|| None),
        |raw_value| {
            if raw_value.trim().is_empty() {
                Ok(None)
            } else {
                Ipv4Addr::from_str(&raw_value)
                    .map(Some)
                    .map_err(|_| "Invalid IP address format, expected XXX.XXX.XXX.XXX".to_owned())
            }
        },
    );

    let state_changed = props.state_changed.clone();

    let update_state = {
        let dhcp_enabled = dhcp_enabled.clone();
        let subnet = subnet.clone();
        let ip = ip.clone();
        let dns = dns.clone();
        let secondary_dns = secondary_dns.clone();

        move |()| {
            let has_errors = !disabled
                && (dhcp_enabled.has_errors()
                    || dhcp_enabled.value() != Some(true)
                        && (subnet.has_errors()
                            || ip.has_errors()
                            || dns.has_errors()
                            || secondary_dns.has_errors()));

            let state = if has_errors {
                ClientState::Errors
            } else {
                let is_dirty = dhcp_enabled.is_dirty()
                    || dhcp_enabled.value() != Some(true)
                        && (subnet.is_dirty()
                            || ip.is_dirty()
                            || dns.is_dirty()
                            || secondary_dns.is_dirty());

                if !is_dirty {
                    ClientState::Unchanged
                } else {
                    ClientState::Conf(if dhcp_enabled.value().unwrap() {
                        ipv4::ClientConfiguration::DHCP(DHCPClientSettings { hostname: None })
                    } else {
                        ipv4::ClientConfiguration::Fixed(ipv4::ClientSettings {
                            subnet: subnet.value().unwrap(),
                            ip: ip.value().unwrap(),
                            dns: dns.value().unwrap(),
                            secondary_dns: secondary_dns.value().unwrap(),
                        })
                    })
                }
            };

            state_changed.emit(state);
        }
    };

    let update_state = Callback::from(update_state);

    let hidden = if_true(disabled, "visibility: hidden;");
    let input_class = |errors| classes!("input", if_true(!disabled && errors, "is-danger"));

    html! {
        <>
        // DHCP
        <div class="field">
            <label class="checkbox" disabled={disabled}>
                <input
                    type="checkbox"
                    checked={dhcp_enabled.raw_value()}
                    disabled={disabled}
                    onclick={dhcp_enabled.change(update_state.clone())}
                />
                {"DHCP"}
            </label>
        </div>

        {
            if !dhcp_enabled.value().unwrap_or(true) {
                html! {
                    <>
                    // Gateway/Subnet
                    <div class="field">
                        <label class="label">{ "Gateway/Subnet" }</label>
                        <div class="control">
                            <input
                                class={input_class(subnet.has_errors())}
                                type="text"
                                placeholder="XXX.XXX.XXX.XXX/YY"
                                value={subnet.raw_value()}
                                disabled={disabled}
                                oninput={subnet.change(update_state.clone())}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{subnet.error_str()}</p>
                    </div>

                    // IP
                    <div class="field">
                        <label class="label">{ "IP" }</label>
                        <div class="control">
                            <input
                                class={input_class(ip.has_errors())}
                                type="text"
                                placeholder="XXX.XXX.XXX.XXX"
                                value={ip.raw_value()}
                                disabled={disabled}
                                oninput={ip.change(update_state.clone())}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{ip.error_str()}</p>
                    </div>

                    // DNS
                    <div class="field">
                        <label class="label">{ "DNS" }</label>
                        <div class="control">
                            <input
                                class={input_class(dns.has_errors())}
                                type="text"
                                placeholder="XXX.XXX.XXX.XXX (optional)"
                                value={dns.raw_value()}
                                disabled={disabled}
                                oninput={dns.change(update_state.clone())}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{dns.error_str()}</p>
                    </div>

                    // Secondary DNS
                    <div class="field">
                        <label class="label">{ "Secondary DNS" }</label>
                        <div class="control">
                            <input
                                class={input_class(secondary_dns.has_errors())}
                                type="text"
                                placeholder="XXX.XXX.XXX.XXX (optional)"
                                value={secondary_dns.raw_value()}
                                disabled={disabled}
                                oninput={secondary_dns.change(update_state)}
                                />
                        </div>
                        <p class="help is-danger" style={hidden}>{secondary_dns.error_str()}</p>
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
