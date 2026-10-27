use crate::ip_service::{fetch_ip_info, format_plain_text};
use crate::models::IpCache;
use reqwest::Client;

pub async fn handle_telegram_webhook(
    update: crate::models::TelegramUpdate,
    bot_token: String,
    cache: IpCache, // <-- Передаем кеш
) -> Result<(), String> {
    let message = match update.message {
        Some(msg) => msg,
        None => return Ok(()),
    };

    let chat_id = message.chat.id;
    let text = match message.text {
        Some(t) => t,
        None => return Ok(()),
    };

    let ip = text.trim();

    if !ip.contains('.') && !ip.contains(':') {
        send_telegram_message(
            &bot_token,
            chat_id,
            "⚠️ Пожалуйста, отправьте валидный IP-адрес.",
        )
        .await?;
        return Ok(());
    }

    // Проверяем кеш перед запросом
    let info = if let Some(cached_info) = cache.get(ip).await {
        println!("[Cache HIT] IP: {}", ip);
        cached_info
    } else {
        println!("[Cache MISS] IP: {}", ip);
        match fetch_ip_info(ip).await {
            Ok(info) => {
                // Сохраняем в кеш (например, со сроком жизни, либо просто по размеру)
                cache.insert(ip.to_string(), info.clone()).await;
                info
            }
            Err(err) => {
                let err_msg = format!("❌ Не удалось получить данные по IP `{}`: {}", ip, err);
                send_telegram_message(&bot_token, chat_id, &err_msg).await?;
                return Ok(());
            }
        }
    };

    let reply_text = format_plain_text(&info);
    send_telegram_message(&bot_token, chat_id, &reply_text).await?;
    Ok(())
}

async fn send_telegram_message(token: &str, chat_id: i64, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client = Client::new();

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown"
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send telegram message: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Telegram API error status: {}", resp.status()));
    }

    Ok(())
}
