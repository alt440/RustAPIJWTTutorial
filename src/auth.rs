use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, TokenData};
use std::collections::HashMap;

pub fn create_jwt(username: &str, roles: Vec<String>, secret: &str) -> String {
    let claims = Claims {
        sub: username.to_owned(),
        roles,
        exp: 10000000000, // Set expiration
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
}