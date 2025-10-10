use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::json;
use standard_error::{StandardError, Status};

use crate::{
    pkg::users::{adaptors::register::RegisterActions, models::User},
    prelude::Result,
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegistrationInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

pub async fn initiate_user_registration(
    State(state): State<AppState>,
    Json(payload): Json<RegistrationInput>,
) -> Result<String> {
    if payload.password != payload.confirm_password {
        return Err(StandardError::new("ERR-REG-000").code(StatusCode::BAD_REQUEST));
    }
    let user = User::new(payload);
    user.save(&state.db_pool).await?;
    user.initiate_registration(state).await?;
    Ok(serde_json::to_string(&json!({ "msg": "registration initiated successfully" }))?)
}

#[derive(Deserialize)]
pub struct RegVerifyInput{
    pub code: String
}

pub async fn verify_user_registration(
    State(state): State<AppState>,
    Query(params): Query<RegVerifyInput>
) -> Result<String>{
    User::verify_registration(state, &params.code).await?;
    Ok(serde_json::to_string(&json!({ "msg": "user verified successfully" }))?)
}
