use crate::{
    constants::utility::log_query, services::google_cloud_authentication::GoogleCloudAuthentication,
};
use actix_web::{web, Error};
use models::test_models::TestRequest;

use crate::models;

pub async fn test_route(body: web::Json<TestRequest>) -> Result<String, Error> {
    log_query("Test route called");
    println!("{:?}", body);
    Ok(format!("Hello World"))
}

pub async fn test_google_auth_route() -> Result<String, Error> {
    log_query("Test Google Auth route called");

    let client = GoogleCloudAuthentication::get_authenticated_client()
        .await
        .unwrap();

    println!("{:?}", client);

    Ok(format!("Hello World"))
}
