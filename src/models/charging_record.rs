use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::ChargingMode;
use sqlx::Row;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingRecord {
    pub id: Uuid,                    // 详单ID
    pub user_id: Uuid,              // 用户ID
    pub pile_id: String,            // 充电桩编号
    pub mode: ChargingMode,         // 充电模式
    pub charging_amount: f64,       // 充电量（度）
    pub charging_time: f64,         // 充电时长（小时）
    pub charging_fee: f64,          // 充电费用
    pub service_fee: f64,           // 服务费用
    pub total_fee: f64,             // 总费用
    pub start_time: NaiveDateTime,  // 开始时间
    pub end_time: NaiveDateTime,    // 结束时间
    pub created_at: NaiveDateTime,  // 详单生成时间
}

impl ChargingRecord {
    pub fn new(
        user_id: Uuid,
        pile_id: String,
        mode: ChargingMode,
        charging_amount: f64,
        charging_time: f64,
        charging_fee: f64,
        service_fee: f64,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            pile_id,
            mode,
            charging_amount,
            charging_time,
            charging_fee,
            service_fee,
            total_fee: charging_fee + service_fee,
            start_time: start_time.naive_utc(),
            end_time: end_time.naive_utc(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    // 根据 user_id 获取所有充电详单
    pub async fn find_by_user_id(user_id: Uuid, pool: &sqlx::MySqlPool) -> Result<Vec<Self>, sqlx::Error> {
        // 将 UUID 转换为字节数组用于查询
        let user_id_bytes = user_id.as_bytes().to_vec();
        
        let rows = sqlx::query(
            r#"
            SELECT 
                id, 
                user_id, 
                pile_id, 
                mode, 
                charging_amount, 
                charging_time, 
                charging_fee, 
                service_fee, 
                total_fee, 
                start_time, 
                end_time, 
                created_at
            FROM charging_records
            WHERE user_id = ?
            "#,
        )
        .bind(user_id_bytes)
        .fetch_all(pool)
        .await?;

        let mut records = Vec::new();
        for row in rows {
            // 从字节数组转换回 UUID
            let id_bytes: Vec<u8> = row.get("id");
            let user_id_bytes: Vec<u8> = row.get("user_id");
            
            let id = Uuid::from_slice(&id_bytes).map_err(|e| {
                sqlx::Error::Decode(format!("Failed to decode UUID: {}", e).into())
            })?;
            
            let user_id = Uuid::from_slice(&user_id_bytes).map_err(|e| {
                sqlx::Error::Decode(format!("Failed to decode UUID: {}", e).into())
            })?;
            
            // 处理 ChargingMode 枚举
            let mode_str: String = row.get("mode");
            let mode = match mode_str.as_str() {
                "Fast" => ChargingMode::Fast,
                "Slow" => ChargingMode::Slow,
                _ => return Err(sqlx::Error::Decode("Invalid charging mode".into())),
            };

            records.push(ChargingRecord {
                id,
                user_id,
                pile_id: row.get("pile_id"),
                mode,
                charging_amount: row.get("charging_amount"),
                charging_time: row.get("charging_time"),
                charging_fee: row.get("charging_fee"),
                service_fee: row.get("service_fee"),
                total_fee: row.get("total_fee"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                created_at: row.get("created_at"),
            });
        }

        Ok(records)
    }
} 