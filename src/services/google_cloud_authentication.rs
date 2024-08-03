use base64::{engine::general_purpose, Engine};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};

pub struct GoogleCloudAuthentication;

impl GoogleCloudAuthentication {
    /// Creates a Google Cloud Authenticated Client
    /// # Usage
    /// ```
    /// use crate::services::google_cloud_authentication::GoogleCloudAuthentication;
    /// let client = GoogleCloudAuthentication::get_authenticated_client().await.unwrap();
    /// ```
    pub async fn get_authenticated_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
        let service_account_base64 = std::env::var("SERVICE_ACCOUNT").unwrap();
        let service_account_bytes = general_purpose::STANDARD
            .decode(&service_account_base64)
            .unwrap();

        let decoded_string = String::from_utf8(service_account_bytes).unwrap();
        let service_account_private_key_base64 =
            std::env::var("SERVICE_ACCOUNT_PRIVATE_KEY").unwrap();
        let service_account_private_key_bytes = general_purpose::STANDARD
            .decode(&service_account_private_key_base64)
            .unwrap();
        let service_account_private_key =
            String::from_utf8(service_account_private_key_bytes).unwrap();
        let sanitized_service_account = Self::sanitize_base64_string(&decoded_string);
        let mut json: Value =
            serde_json::from_str(&sanitized_service_account).expect("Invalid JSON");

        json["private_key"] = serde_json::Value::String(service_account_private_key);

        let sa_key = ServiceAccountKey {
            key_type: Some(
                json["type"]
                    .as_str()
                    .unwrap_or("service_account")
                    .to_string(),
            ),
            project_id: Some(json["project_id"].as_str().unwrap_or("").to_string()),
            private_key_id: Some(json["private_key_id"].as_str().unwrap_or("").to_string()),
            private_key: json["private_key"].as_str().unwrap_or("").to_string(),
            client_email: json["client_email"].as_str().unwrap_or("").to_string(),
            client_id: Some(json["client_id"].as_str().unwrap_or("").to_string()),
            auth_uri: Some(
                json["auth_uri"]
                    .as_str()
                    .unwrap_or("https://accounts.google.com/o/oauth2/auth")
                    .to_string(),
            ),
            token_uri: json["token_uri"]
                .as_str()
                .unwrap_or("https://oauth2.googleapis.com/token")
                .to_string(),
            auth_provider_x509_cert_url: Some(
                json["auth_provider_x509_cert_url"]
                    .as_str()
                    .unwrap_or("https://www.googleapis.com/oauth2/v1/certs")
                    .to_string(),
            ),
            client_x509_cert_url: Some(
                json["client_x509_cert_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            ),
        };

        let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;
        let token = auth
            .token(&["https://www.googleapis.com/auth/cloud-vision"])
            .await?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.token().unwrap()))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(client)
    }

    fn sanitize_base64_string(base64_str: &str) -> String {
        base64_str.replace("\n", "").replace("\r", "")
    }
}
