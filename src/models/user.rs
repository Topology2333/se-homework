use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::MySql;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, password_hash: String, is_admin: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            is_admin,
            created_at: Utc::now(),
        }
    }

    pub async fn insert(&self, pool: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, password_hash, is_admin, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            self.id,
            self.username,
            self.password_hash,
            self.is_admin,
            self.created_at
        )
        .execute(pool)
        .await?;
        Ok(())
    }

pub async fn get_all(pool: &sqlx::MySqlPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<MySql, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users"
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

}
