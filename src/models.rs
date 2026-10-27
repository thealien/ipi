use moka::future::Cache;
use serde::{Deserialize, Serialize};

// Кеш: ключ — строка (IP), значение — готовая структура IpInfoResponse
pub type IpCache = Cache<String, IpInfoResponse>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoInfo {
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub city: String,
    pub postal: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInfo {
    pub asn: u32,
    pub asn_org: String,
    pub isp: String,
    pub organization: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityInfo {
    pub connection_type: String,
    pub is_vpn: bool,
    pub is_proxy: bool,
    pub is_tor: bool,
    pub is_hosting: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TechnicalInfo {
    pub ip_version: u8,
    pub hostname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpInfoResponse {
    pub ip: String,
    pub geo: GeoInfo,
    pub network: NetworkInfo,
    pub security: SecurityInfo,
    pub technical: TechnicalInfo,
}

// Структуры для Telegram Webhook
#[derive(Deserialize, Debug)]
pub struct TelegramUpdate {
    pub message: Option<TelegramMessage>,
}

#[derive(Deserialize, Debug)]
pub struct TelegramMessage {
    pub chat: TelegramChat,
    pub text: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TelegramChat {
    pub id: i64,
}
