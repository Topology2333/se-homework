use crate::models::{ChargingMode, RequestStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Type};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChargingRequest {
    pub id: Uuid,                  // 请求ID
    pub user_id: Uuid,             // 用户ID
    pub mode: String,              // 充电模式
    pub amount: f64,               // 请求充电量（度）
    pub queue_number: String,      // 排队号码（F1、F2、T1、T2等）
    pub status: String,            // 请求状态
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

impl ChargingRequest {
    pub fn new(user_id: Uuid, mode: ChargingMode, amount: f64, queue_number: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            mode: mode.to_string(),
            amount,
            queue_number,
            status: RequestStatus::Waiting.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 开始充电
    pub fn start_charging(&mut self) -> Result<(), String> {
        let status = RequestStatus::from_str(&self.status)?;
        match status {
            RequestStatus::Waiting => {
                self.status = RequestStatus::Charging.to_string();
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    /// 完成充电
    pub fn complete_charging(&mut self) -> Result<(), String> {
        let status = RequestStatus::from_str(&self.status)?;
        match status {
            RequestStatus::Charging => {
                self.status = RequestStatus::Completed.to_string();
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    /// 取消请求
    pub fn cancel(&mut self) -> Result<(), String> {
        let status = RequestStatus::from_str(&self.status)?;
        match status {
            RequestStatus::Waiting | RequestStatus::Charging => {
                self.status = RequestStatus::Cancelled.to_string();
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    pub fn update_amount(&mut self, new_amount: f64) {
        self.amount = new_amount;
    }

    pub fn update_mode(&mut self, new_mode: ChargingMode, new_queue_number: String) {
        self.mode = new_mode.to_string();
        self.queue_number = new_queue_number;
        self.updated_at = Utc::now();
    }

    // 数据库操作方法
    /// 创建新的充电请求
    pub async fn create(&self, pool: &MySqlPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO charging_requests (id, user_id, mode, amount, queue_number, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.user_id,
            self.mode.to_string(),
            self.amount,
            self.queue_number,
            self.status.to_string(),
            self.created_at,
            self.updated_at
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 根据ID查询充电请求
    pub async fn get_by_id(
        pool: &MySqlPool,
        id: Uuid,
    ) -> Result<Option<ChargingRequest>, sqlx::Error> {
        sqlx::query_as::<_, ChargingRequest>(
            r#"
            SELECT 
                id,
                user_id,
                mode,
                amount,
                queue_number,
                status,
                created_at,
                updated_at
            FROM charging_requests
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
    }

    /// 根据用户ID查询充电请求
    pub async fn get_by_user_id(
        pool: &MySqlPool,
        user_id: Uuid,
    ) -> Result<Vec<ChargingRequest>, sqlx::Error> {
        sqlx::query_as::<_, ChargingRequest>(
            r#"
            SELECT 
                id,
                user_id,
                mode,
                amount,
                queue_number,
                status,
                created_at,
                updated_at
            FROM charging_requests
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(pool)
        .await
    }

    /// 获取指定状态的充电请求
    pub async fn get_by_status(
        pool: &MySqlPool,
        status: RequestStatus,
    ) -> Result<Vec<ChargingRequest>, sqlx::Error> {
        sqlx::query_as::<_, ChargingRequest>(
            r#"
        SELECT 
            id,
            user_id,
            mode,
            amount,
            queue_number,
            status,
            created_at,
            updated_at
        FROM charging_requests
        WHERE status = ?
        ORDER BY created_at ASC
        "#,
        )
        .bind(status.to_string()) // ✅ 将枚举转换为字符串再绑定
        .fetch_all(pool)
        .await
    }

    /// 获取指定模式和状态的充电请求队列
    pub async fn get_queue(
        pool: &MySqlPool,
        mode: ChargingMode,
        status: RequestStatus,
    ) -> Result<Vec<ChargingRequest>, sqlx::Error> {
        sqlx::query_as::<_, ChargingRequest>(
            r#"
        SELECT
            id,
            user_id,
            mode,
            amount,
            queue_number,
            status,
            created_at,
            updated_at
        FROM charging_requests
        WHERE mode = ? AND status = ?
        ORDER BY created_at ASC
        "#,
        )
        .bind(mode.to_string())
        .bind(status.to_string())
        .fetch_all(pool)
        .await
    }

    /// 更新充电请求状态
    pub async fn update_status(
        &mut self,
        pool: &MySqlPool,
        new_status: RequestStatus,
    ) -> Result<(), sqlx::Error> {
        self.status = new_status.to_string();
        self.updated_at = Utc::now();

        sqlx::query!(
            r#"
            UPDATE charging_requests
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
            self.status.to_string(),
            self.updated_at,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 更新充电请求信息
    pub async fn update(&mut self, pool: &MySqlPool) -> Result<(), sqlx::Error> {
        self.updated_at = Utc::now();

        sqlx::query!(
            r#"
            UPDATE charging_requests
            SET mode = ?, amount = ?, queue_number = ?, status = ?, updated_at = ?
            WHERE id = ?
            "#,
            self.mode.to_string(),
            self.amount,
            self.queue_number,
            self.status.to_string(),
            self.updated_at,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 删除充电请求
    pub async fn delete(&self, pool: &MySqlPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM charging_requests
            WHERE id = ?
            "#,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request() {
        let user_id = Uuid::new_v4();
        let request = ChargingRequest::new(user_id, ChargingMode::Fast, 30.0, "F1".to_string());

        assert_eq!(request.user_id, user_id);
        assert_eq!(request.mode, ChargingMode::Fast);
        assert_eq!(request.amount, 30.0);
        assert_eq!(request.queue_number, "F1");
        assert_eq!(request.status, RequestStatus::Waiting);
    }

    #[test]
    fn test_request_lifecycle() {
        let mut request =
            ChargingRequest::new(Uuid::new_v4(), ChargingMode::Fast, 30.0, "F1".to_string());

        // 开始充电
        request.start_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Charging);

        // 完成充电
        request.complete_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Completed);
    }

    #[test]
    fn test_cancel_request() {
        let mut request =
            ChargingRequest::new(Uuid::new_v4(), ChargingMode::Fast, 30.0, "F1".to_string());

        // 等待状态下取消
        request.cancel().unwrap();
        assert_eq!(request.status, RequestStatus::Cancelled);

        // 已取消状态下不能再取消
        assert!(request.cancel().is_err());
    }
}
