use serde::{Deserialize, Serialize};
use crate::config::ChannelConfig;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPayload {
    pub title: String,
    pub content: String,
}

pub async fn send_feishu(
    client: &reqwest::Client,
    webhook_url: &str,
    _secret: Option<String>,
    payload: &NotificationPayload,
) -> Result<(), String> {
    let formatted_text = format!("【{}】\n{}", payload.title, payload.content);
    let body = serde_json::json!({
        "msg_type": "text",
        "content": {
            "text": formatted_text
        }
    });

    let response = client
        .post(webhook_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("飞书 Webhook 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_err = response.text().await.unwrap_or_default();
        return Err(format!("飞书接口返回状态 {}: {}", status, body_err));
    }

    Ok(())
}

pub async fn send_server_chan(
    client: &reqwest::Client,
    send_key: &str,
    payload: &NotificationPayload,
) -> Result<(), String> {
    let url = format!("https://sctapi.ftqq.com/{}.send", send_key);
    let body = serde_json::json!({
        "title": payload.title,
        "desp": payload.content
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Server酱 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_err = response.text().await.unwrap_or_default();
        return Err(format!("Server酱接口返回状态 {}: {}", status, body_err));
    }

    Ok(())
}

pub async fn send_server_chan3(
    client: &reqwest::Client,
    uid: &str,
    send_key: &str,
    payload: &NotificationPayload,
) -> Result<(), String> {
    let url = format!("https://{}.push.ft07.com/send/{}.send", uid, send_key);
    let body = serde_json::json!({
        "title": payload.title,
        "desp": payload.content
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Server酱³ 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_err = response.text().await.unwrap_or_default();
        return Err(format!("Server酱³接口返回状态 {}: {}", status, body_err));
    }

    Ok(())
}

pub async fn send_telegram(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: &str,
    payload: &NotificationPayload,
) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let text = format!("<b>{}</b>\n{}", payload.title, payload.content);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML"
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Telegram 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_err = response.text().await.unwrap_or_default();
        return Err(format!("Telegram接口返回状态 {}: {}", status, body_err));
    }

    Ok(())
}

pub async fn send_notification(
    config: &ChannelConfig,
    payload: &NotificationPayload,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    match config {
        ChannelConfig::Feishu { webhook_url, secret } => {
            send_feishu(&client, webhook_url, secret.clone(), payload).await
        }
        ChannelConfig::ServerChan { send_key } => {
            send_server_chan(&client, send_key, payload).await
        }
        ChannelConfig::ServerChan3 { uid, send_key } => {
            send_server_chan3(&client, uid, send_key, payload).await
        }
        ChannelConfig::Telegram { bot_token, chat_id } => {
            send_telegram(&client, bot_token, chat_id, payload).await
        }
    }
}


#[tauri::command]
pub async fn send_aggregated_notification(
    channels: Vec<ChannelConfig>,
    payload: NotificationPayload,
) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut success_count = 0;

    for config in channels {
        match send_notification(&config, &payload).await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                errors.push(format!("{:?}: {}", config, e));
            }
        }
    }

    if errors.is_empty() {
        Ok(vec![format!("成功通过 {} 个通道发送通知", success_count)])
    } else {
        Err(errors.join("; "))
    }
}

pub fn trigger_task_notification(
    app_handle: &tauri::AppHandle,
    task_id: &str,
    task_name: &str,
    status: &str, // "completed" or "interrupted"
    message: &str,
) {
    let handle = app_handle.clone();
    let tid = task_id.to_string();
    let tname = task_name.to_string();
    let status_str = status.to_string();
    let msg_str = message.to_string();

    tokio::spawn(async move {
        // 1. 获取 TaskQueue 并读取任务的 notification_channels
        let task_channels = {
            if let Some(queue) = handle.try_state::<crate::scheduler::TaskQueue>() {
                if let Some(task) = queue.get_task(&tid) {
                    task.notification_channels
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        if task_channels.is_empty() {
            return;
        }

        // 2. 获取 AppConfig 并读取匹配的全局通知通道配置
        let matched_channels = {
            if let Some(config_mgr) = handle.try_state::<crate::config::AppConfigManager>() {
                let config = config_mgr.get();
                config
                    .notification_channels
                    .iter()
                    .filter(|c| task_channels.contains(&c.id))
                    .map(|c| c.config.clone())
                    .collect::<Vec<ChannelConfig>>()
            } else {
                return;
            }
        };

        if matched_channels.is_empty() {
            return;
        }

        // 3. 构建载荷
        let emoji = if status_str == "completed" { "✅" } else { "⚠️" };
        let status_cn = if status_str == "completed" { "已完成" } else { "已中断/失败" };
        let title = format!("{} AutoController 任务通知 - {}", emoji, tname);
        let content = format!(
            "任务名称：{}\n执行状态：{}\n详细信息：{}\n通知时间：{}",
            tname,
            status_cn,
            msg_str,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let payload = NotificationPayload { title, content };

        // 4. 异步发送
        for config in matched_channels {
            if let Err(e) = send_notification(&config, &payload).await {
                tracing::error!(task_id = %tid, error = %e, "发送任务通知失败");
            }
        }
    });
}
