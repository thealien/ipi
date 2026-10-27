use crate::models::*;

pub async fn fetch_ip_info(ip: &str) -> Result<IpInfoResponse, String> {
    // Используем ipwho.is — бесплатный публичный API без ключа
    let url = format!("https://ipwho.is/{}", ip);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API returned status: {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Проверяем статус успешности ответа ipwho.is
    if json["success"].as_bool() == Some(false) {
        let message = json["message"].as_str().unwrap_or("Unknown error");
        return Err(format!("API error: {}", message));
    }

    // Безопасно извлекаем данные или подставляем заглушки
    let ip_str = json["ip"].as_str().unwrap_or(ip).to_string();
    let ip_ver = if ip_str.contains(':') { 6 } else { 4 };

    let geo = GeoInfo {
        country: json["country"].as_str().unwrap_or("N/A").to_string(),
        country_code: json["country_code"].as_str().unwrap_or("N/A").to_string(),
        region: json["region"].as_str().unwrap_or("N/A").to_string(),
        city: json["city"].as_str().unwrap_or("N/A").to_string(),
        postal: json["postal"].as_str().unwrap_or("N/A").to_string(),
        timezone: json["timezone"]["id"].as_str().unwrap_or("N/A").to_string(),
        latitude: json["latitude"].as_f64().unwrap_or(0.0),
        longitude: json["longitude"].as_f64().unwrap_or(0.0),
    };

    let connection = &json["connection"];
    let network = NetworkInfo {
        asn: connection["asn"].as_u64().unwrap_or(0) as u32,
        asn_org: connection["domain"].as_str().unwrap_or("N/A").to_string(),
        isp: connection["isp"].as_str().unwrap_or("N/A").to_string(),
        organization: connection["org"].as_str().unwrap_or("N/A").to_string(),
    };

    let security_block = &json["security"];
    let security = SecurityInfo {
        connection_type: json["type"].as_str().unwrap_or("unknown").to_string(),
        is_vpn: security_block["vpn"].as_bool().unwrap_or(false),
        is_proxy: security_block["proxy"].as_bool().unwrap_or(false),
        is_tor: security_block["tor"].as_bool().unwrap_or(false),
        is_hosting: security_block["hosting"].as_bool().unwrap_or(false),
    };

    let technical = TechnicalInfo {
        ip_version: ip_ver,
        hostname: json["connection"]["domain"]
            .as_str()
            .unwrap_or("N/A")
            .to_string(),
    };

    Ok(IpInfoResponse {
        ip: ip_str,
        geo,
        network,
        security,
        technical,
    })
}

/// Форматирование структуры в красивый plain-текст для Telegram или ответа API
pub fn format_plain_text(info: &IpInfoResponse) -> String {
    format!(
        "🌐 **Информация по IP:** `{}`\n\n\
        📍 **Геолокация**\n\
        • Страна: {} ({})\n\
        • Регион: {}, {}\n\
        • Индекс: {}\n\
        • Часовой пояс: `{}`\n\
        • Координаты: `{}`, `{}`\n\n\
        🏢 **Сеть и провайдер**\n\
        • Провайдер (ISP): {}\n\
        • Организация: {}\n\
        • ASN: `AS{}`\n\n\
        🛡 **Безопасность и тип**\n\
        • Тип сети: `{}`\n\
        • VPN: {}\n\
        • Proxy: {}\n\
        • Tor: {}\n\
        • Хостинг: {}\n\n\
        ⚙️ **Техническая информация**\n\
        • Версия IP: IPv{}\n\
        • Хост: `{}`",
        info.ip,
        info.geo.country,
        info.geo.country_code,
        info.geo.region,
        info.geo.city,
        info.geo.postal,
        info.geo.timezone,
        info.geo.latitude,
        info.geo.longitude,
        info.network.isp,
        info.network.organization,
        info.network.asn,
        info.security.connection_type,
        if info.security.is_vpn {
            "✅ Да"
        } else {
            "❌ Нет"
        },
        if info.security.is_proxy {
            "✅ Да"
        } else {
            "❌ Нет"
        },
        if info.security.is_tor {
            "✅ Да"
        } else {
            "❌ Нет"
        },
        if info.security.is_hosting {
            "✅ Да"
        } else {
            "❌ Нет"
        },
        info.technical.ip_version,
        info.technical.hostname
    )
}
