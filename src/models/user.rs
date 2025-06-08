use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,                // 用户ID
    pub username: String,        // 用户名
    pub password_hash: String,   // 密码哈希
    pub is_admin: bool,         // 是否是管理员
    pub created_at: chrono::DateTime<chrono::Utc>,  // 创建时间
}

impl User {
    pub fn new(username: String, password_hash: String, is_admin: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            is_admin,
            created_at: chrono::Utc::now(),
        }
    }
} 