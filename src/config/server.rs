use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
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
struct Inbound {
    port: i64,
    protocol: String,
    settings: InBoundSettings,
    #[serde(alias = "streamSettings")]
    stream_settings: StreamSettings,
    sniffing: Sniffing,
}

#[derive(Serialize, Deserialize)]
struct Log {
    #[serde(alias = "domainStrategy")]
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
struct OutBoundSettings {
    #[serde(alias = "domainStrategy")]
    domain_strategy: String,
}

#[derive(Serialize, Deserialize)]
struct Sniffing {
    enabled: bool,
    #[serde(alias = "destOverride")]
    dest_override: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct StreamSettings {
    network: String,
    #[serde(alias = "wsSettings")]
    ws_settings: WsSettings,
}

#[derive(Serialize, Deserialize)]
struct WsSettings {
    path: String,
}

impl ServerConfig {
    pub fn new<T: Into<String>>(uuid: T, path: T) -> Self {
        ServerConfig {
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
                    ws_settings: WsSettings { path: path.into() },
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
