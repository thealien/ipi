# 🌐 ipi (IP Info Bot & API)

[![AI Generated](https://img.shields.io/badge/Code%20&%20Docs-Generated%20by%20Gemini-blue?style=flat-square&logo=google)](https://deepmind.google/technologies/gemini/)

Учебный микросервис и Telegram-бот для получения информации об IP-адресах, написанный на Rust...

**ipi** — это высокопроизводительный асинхронный HTTP-сервис и Telegram-бот на языке Rust для получения исчерпывающей информации об IP-адресах (геолокация, провайдер/ASN, параметры безопасности и сетевые флаги).

Проект поддерживает автоматическое in-memory кэширование результатов с помощью библиотека **Moka**, что минимизирует количество внешних сетевых запросов и предотвращает превышение лимитов публичных API.

---

## 🚀 Особенности

- **Высокая скорость работы:** Написан на Rust с использованием асинхронного рантайма `tokio` и веб-фреймворка `axum`.
- **Два формата ответа:**
  - `JSON` (по умолчанию) — структурированные данные для других сервисов.
  - `Plain Text / Markdown` — читаемый текстовый формат с эмодзи для терминала или сообщений Telegram.
- **Интеграция с Telegram:** Поддержка входящих сообщений через Webhook API.
- **Встроенное кеширование:** Автоматический кэш (TTL: 1 час, до 10 000 записей) для исключения повторных запросов к одному и тому же IP.
- **Бесплатный источник данных:** Использование открытого API `ipwho.is` (без необходимости получать API-ключи).

---

## 📊 Структура данных (4 Блока)

Сервис агрегирует информацию по 4 ключевым направлениям:

1. **`geo` (Геолокация):** Страна, код страны, регион, город, почтовый индекс, часовой пояс, GPS-координаты.
2. **`network` (Сеть и провайдер):** ASN, название автономной системы, ISP, организация.
3. **`security` (Безопасность):** Тип подключения (`datacenter`, `residential`, etc.), флаги использования VPN, Proxy, Tor, Hosting.
4. **`technical` (Технические параметры):** Версия IP (v4 / v6), PTR-запись (Reverse DNS / Hostname).

### Пример ответа JSON (`GET /ipinfo?ip=8.8.8.8`)

```json
{
  "ip": "8.8.8.8",
  "geo": {
    "country": "United States",
    "country_code": "US",
    "region": "California",
    "city": "Mountain View",
    "postal": "94043",
    "timezone": "America/Los_Angeles",
    "latitude": 37.4056,
    "longitude": -122.0775
  },
  "network": {
    "asn": 15169,
    "asn_org": "GOOGLE",
    "isp": "Google LLC",
    "organization": "Google Public DNS"
  },
  "security": {
    "connection_type": "datacenter",
    "is_vpn": false,
    "is_proxy": false,
    "is_tor": false,
    "is_hosting": true
  },
  "technical": {
    "ip_version": 4,
    "hostname": "dns.google"
  }
}
```

---

## 🛠 Требования для сборки

- **Rust** (компилятор версии 1.70+ и пакетный менеджер `cargo`).
- Доступ в Интернет для скачивания зависимостей и обращения к API.

---

## 💻 Локальный запуск

1. **Клонируйте репозиторий:**
   ```bash
   git clone https://github.com/your-username/ipi.git
   cd ipi
   ```

2. **Создайте файл переменной окружения `.env` в корне проекта:**
   ```env
   PORT=3000
   TELEGRAM_BOT_TOKEN=your_telegram_bot_token_here
   ```

3. **Запустите проект:**
   ```bash
   cargo run
   ```

4. **Проверьте работу эндпоинта:**
   - **JSON format:** [http://localhost:3000/ipinfo?ip=8.8.8.8](http://localhost:3000/ipinfo?ip=8.8.8.8)
   - **Plain text format:** [http://localhost:3000/ipinfo?ip=8.8.8.8&format=plain](http://localhost:3000/ipinfo?ip=8.8.8.8&format=plain)

---

## ☁️ Развертывание на VPS (Debian / Ubuntu)

### Шаг 1. Подготовка сервера и установка Rust

```bash
# Обновление пакетов
sudo apt update && sudo apt upgrade -y
sudo apt install curl build-essential git -y

# Установка Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Шаг 2. Сборка приложения

```bash
# Скопируйте исходный код в /opt/ipi и перейдите в директорию
cd /opt/ipi

# Сборка бинарного файла с оптимизациями релиза
cargo build --release
```

Бинарный файл появится по пути: `/opt/ipi/target/release/ipi`.

### Шаг 3. Настройка фонового сервиса (Systemd)

Создайте файл службы:

```bash
sudo nano /etc/systemd/system/ipi.service
```

Вставьте следующую конфигурацию (замените `YOUR_TELEGRAM_BOT_TOKEN` на ваш токен):

```ini
[Unit]
Description=ipi - IP Info API Service & Telegram Bot
After=network.target

[Service]
User=root
WorkingDirectory=/opt/ipi
ExecStart=/opt/ipi/target/release/ipi
Restart=always
RestartSec=5
Environment=PORT=3000
Environment=TELEGRAM_BOT_TOKEN=YOUR_TELEGRAM_BOT_TOKEN

[Install]
WantedBy=multi-user.target
```

Перезагрузите демон systemd и запустите службу:

```bash
sudo systemctl daemon-reload
sudo systemctl start ipi
sudo systemctl enable ipi
```

Проверить статус:
```bash
sudo systemctl status ipi
```

### Шаг 4. Настройка Telegram Webhook

Настройте Webhook в Telegram API, указав IP вашего сервера или привязанный домен:

```bash
curl -F "url=http://<YOUR_VPS_IP>:3000/telegram/webhook" https://api.telegram.org/bot<YOUR_TELEGRAM_BOT_TOKEN>/setWebhook
```

> **Примечание:** Для работы Webhook Telegram в продакшене рекомендуется использовать доменное имя с настроенным SSL/HTTPS (например, настроив обратный прокси-сервер Nginx + Certbot).
