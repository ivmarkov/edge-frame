use std::net::Ipv4Addr;
use std::str::FromStr;

use yew::prelude::*;

use embedded_svc::ipv4::{self, RouterConfiguration, Subnet};

use crate::field::*;
use crate::util::*;

pub type RouterConf = ipv4::RouterConfiguration;

pub enum RouterState {
    Unchanged,
    Errors,
    Conf(RouterConf),
}

impl RouterState {
    pub fn conf(&self) -> Option<&RouterConf> {
        if let Self::Conf(conf) = self {
            Some(conf)
        } else {
            None
        }
    }
}

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct RouterProps {
    #[prop_or_default]
    pub conf: RouterConf,

    #[prop_or_default]
    pub disabled: bool,

    pub state_changed: Callback<RouterState>,
}

#[function_component(Router)]
pub fn router(props: &RouterProps) -> Html {
    let conf = &props.conf;
    let disabled = props.disabled;

    let dhcp_server_enabled = Field::checked(conf.dhcp_enabled, use_state(|| None), Ok);
    let subnet = Field::text(conf.subnet.to_string(), use_state(|| None), |raw_text| {
        Subnet::from_str(&raw_text).map_err(str::to_owned)
    });
    let dns = TextField::<Option<Ipv4Addr>>::text(
        conf.dns.map(|d| d.to_string()).unwrap_or_default(),
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
        conf.secondary_dns
            .map(|d| d.to_string())
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
        let dhcp_server_enabled = dhcp_server_enabled.clone();
        let subnet = subnet.clone();
        let dns = dns.clone();
        let secondary_dns = secondary_dns.clone();

        move |()| {
            let has_errors = dhcp_server_enabled.has_errors()
                || subnet.has_errors()
                || dns.has_errors()
                || secondary_dns.has_errors();

            let state = if has_errors {
                RouterState::Errors
            } else {
                let is_dirty = dhcp_server_enabled.is_dirty()
                    || subnet.is_dirty()
                    || dns.is_dirty()
                    || secondary_dns.is_dirty();

                if !is_dirty {
                    RouterState::Unchanged
                } else {
                    RouterState::Conf(RouterConfiguration {
                        dhcp_enabled: dhcp_server_enabled.value().unwrap(),
                        subnet: subnet.value().unwrap(),
                        dns: dns.value().unwrap(),
                        secondary_dns: secondary_dns.value().unwrap(),
                    })
                }
            };

            state_changed.emit(state);
        }
    };

    let update_state = Callback::<(), ()>::from(update_state);

    let hidden = if_true(disabled, "visibility: hidden;");
    let input_class = |errors| classes!("input", if_true(!disabled && errors, "is-danger"));

    html! {
        <>
        // DHCP Server
        <div class="field">
            <label class="checkbox" disabled={disabled}>
                <input
                    type="checkbox"
                    checked={dhcp_server_enabled.raw_value()}
                    disabled={disabled}
                    onclick={dhcp_server_enabled.change(update_state.clone())}
                />
                {"DHCP Server"}
            </label>
        </div>

        // Subnet
        <div class="field">
            <label class="label">{ "Subnet" }</label>
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
}
