use reqwest::Client;
use std::env;

pub struct EmailService {
    client: Client,
    sender_email: String,
    api_key: String,
}

impl EmailService {
    pub fn new() -> Self {
        let sender_email = "admin@neocadmium.softwarescompound.in".to_string();
        let api_key = env::var("RESEND_TOKEN").expect("RESEND_TOKEN must be set");

        Self {
            client: Client::new(),
            sender_email,
            api_key,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let payload = serde_json::json!({
            "from": format!("Neocadmium <{}>", self.sender_email),
            "to": recipient,
            "subject": subject,
            "html": body
        });

        let res = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to send email: {:?}", res.status()))
        }
    }
}
