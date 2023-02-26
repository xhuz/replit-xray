use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct XrayConfig {
    log: Log,
    dns: Dns,
    inbounds: Vec<Inbound>,
    outbounds: Vec<Outbound>,
}

#[derive(Serialize, Deserialize)]
struct Client {
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Dns {
    servers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Inbound {
    port: i64,
    protocol: String,
    settings: InBoundSettings,
    stream_settings: StreamSettings,
    sniffing: Sniffing,
}

#[derive(Serialize, Deserialize)]
struct Log {
    #[serde(rename = "loglevel")]
    log_level: String,
}

#[derive(Serialize, Deserialize)]
struct Outbound {
    protocol: String,
    tag: String,
    settings: OutBoundSettings,
}

#[derive(Serialize, Deserialize)]
struct InBoundSettings {
    clients: Vec<Client>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutBoundSettings {
    domain_strategy: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sniffing {
    enabled: bool,
    dest_override: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamSettings {
    network: String,
    ws_settings: WsSettings,
}

#[derive(Serialize, Deserialize)]
struct WsSettings {
    path: String,
}

impl XrayConfig {
    pub fn new<T: Into<String>>(uuid: T, path: T) -> Self {
        XrayConfig {
            log: Log {
                log_level: "info".to_owned(),
            },
            dns: Dns {
                servers: vec!["https+local://8.8.8.8/dns-query".to_owned()],
            },
            inbounds: vec![Inbound {
                port: 7707,
                protocol: "trojan".to_owned(),
                settings: InBoundSettings {
                    clients: vec![Client {
                        password: uuid.into(),
                    }],
                },
                stream_settings: StreamSettings {
                    network: "ws".to_owned(),
                    ws_settings: WsSettings {
                        path: format!("/{}", path.into()),
                    },
                },
                sniffing: Sniffing {
                    enabled: true,
                    dest_override: vec!["http".to_owned(), "tls".to_owned(), "quic".to_owned()],
                },
            }],
            outbounds: vec![Outbound {
                protocol: "freedom".to_owned(),
                tag: "direct".to_owned(),
                settings: OutBoundSettings {
                    domain_strategy: "UseIPv4".to_owned(),
                },
            }],
        }
    }
}

#[cfg(test)]
mod test {

    use super::XrayConfig;
    use serde_json;

    const JSON: &str = r#"
    {"log":{"loglevel":"info"},"dns":{"servers":["https+local://8.8.8.8/dns-query"]},"inbounds":[{"port":7707,"protocol":"trojan","settings":{"clients":[{"password":"123"}]},"streamSettings":{"network":"ws","wsSettings":{"path":"/123"}},"sniffing":{"enabled":true,"destOverride":["http","tls","quic"]}}],"outbounds":[{"protocol":"freedom","tag":"direct","settings":{"domainStrategy":"UseIPv4"}}]}
    "#;

    #[test]
    fn test_serialize() {
        let x = XrayConfig::new("123", "123");

        let json = serde_json::to_string(&x).unwrap();

        assert_eq!(json, JSON.trim())
    }

    #[test]
    fn test_deserialize() {
        let c = serde_json::from_str::<XrayConfig>(JSON);

        assert!(c.is_ok());
    }
}
