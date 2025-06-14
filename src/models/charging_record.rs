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

    /// 插入充电详单到数据库
    pub async fn insert(&self, pool: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
        // 将 UUID 转换为字节数组
        let id_bytes = self.id.as_bytes().to_vec();
        let user_id_bytes = self.user_id.as_bytes().to_vec();
        
        println!("🔍 准备插入充电详单: ID={}, 用户={}, 充电桩={}", self.id, self.user_id, self.pile_id);
        
        let result = sqlx::query(
            r#"
            INSERT INTO charging_records (
                id, 
                user_id, 
                pile_id, 
                mode, 
                charging_amount, 
                charging_fee, 
                service_fee, 
                total_fee, 
                start_time, 
                end_time, 
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id_bytes)
        .bind(user_id_bytes)
        .bind(&self.pile_id)
        .bind(self.mode.to_string())
        .bind(self.charging_amount)
        .bind(self.charging_fee)
        .bind(self.service_fee)
        .bind(self.total_fee)
        .bind(self.start_time)
        .bind(self.end_time)
        .bind(self.created_at)
        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                println!("✅ 充电详单已保存到数据库: 用户 {}, 充电桩 {}, 充电量 {}度, 总费用 {}元", 
                    self.user_id, self.pile_id, self.charging_amount, self.total_fee);
                Ok(())
            }
            Err(e) => {
                println!("❌ 充电详单保存失败: {}", e);
                Err(e)
            }
        }
    }

    /// 批量插入充电详单
    pub async fn insert_batch(records: &[ChargingRecord], pool: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
        if records.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            INSERT INTO charging_records (
                id, user_id, pile_id, mode, charging_amount, 
                charging_fee, service_fee, total_fee, start_time, end_time, created_at
            ) 
            "#
        );

        query_builder.push_values(records, |mut b, record| {
            b.push_bind(record.id.as_bytes().to_vec())
             .push_bind(record.user_id.as_bytes().to_vec())
             .push_bind(&record.pile_id)
             .push_bind(record.mode.to_string())
             .push_bind(record.charging_amount)
             .push_bind(record.charging_fee)
             .push_bind(record.service_fee)
             .push_bind(record.total_fee)
             .push_bind(record.start_time)
             .push_bind(record.end_time)
             .push_bind(record.created_at);
        });

        let query = query_builder.build();
        query.execute(pool).await?;

        println!("✅ 批量保存 {} 条充电详单到数据库", records.len());

        Ok(())
    }
} 