#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub code: String,
    pub email: Option<String>,
    pub username: Option<String>,
}

impl VerificationRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.code.len() != 6 || !self.code.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid verification code format".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub token: String,
    pub username: String,
}

pub fn generate_token(email: &str, username: &str) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .ok_or("Failed to calculate token expiration")?
        .timestamp();

    let token_payload = format!("{}:{}:{}", email, username, expiration);

    let hash = Sha256::digest(token_payload.as_bytes());

    // Convert to numeric-only by taking the decimal digits of the hash bytes
    let numeric_token = hash
        .iter()
        .map(|b| format!("{:03}", b)) // Pad each byte to 3 digits (000–255)
        .collect::<String>();

    Ok(numeric_token)
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
} 
