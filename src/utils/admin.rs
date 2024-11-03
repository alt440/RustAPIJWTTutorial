use crate::db;
use crate::jwt;

pub fn is_admin(token: &str, secret: &str) -> bool{
    // verifies that validate_jwt does not return any errors (The Ok keyword validates a successful return), and assigns the non-erroneous return to data
    if let Ok(data) = jwt::validate_jwt(token, &secret) {
        // if role contains admin, access granted. Currently holds only 1 index
        // Don't know why I can't simply do a for role in &data.claims.roles... the index appears inexistant
        if let Some(first_role) = &data.claims.roles.get(0) {
            // for some reason, first_role extracted with " in prefix and suffix of string
            if (*first_role).contains(&db::models::Roles::Admin.as_str()) {
                return true;
            }
        }
    }
    return false;
}