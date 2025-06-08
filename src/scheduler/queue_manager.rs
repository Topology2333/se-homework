use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    ChargingMode, ChargingPile, ChargingRequest, PileStatus,
    WAITING_AREA_CAPACITY, PILE_QUEUE_CAPACITY,
};

/// 等候区队列管理器
pub struct QueueManager {
    // 等候区队列
    waiting_queue: RwLock<VecDeque<Arc<ChargingRequest>>>,
    
    // 充电桩队列，key为充电桩ID
    pile_queues: RwLock<HashMap<String, VecDeque<Arc<ChargingRequest>>>>,
    
    // 充电桩状态
    piles: RwLock<HashMap<String, Arc<ChargingPile>>>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            waiting_queue: RwLock::new(VecDeque::new()),
            pile_queues: RwLock::new(HashMap::new()),
            piles: RwLock::new(HashMap::new()),
        }
    }

    // 添加充电桩
    pub async fn add_pile(&self, pile: Arc<ChargingPile>) {
        let mut piles = self.piles.write().await;
        let mut pile_queues = self.pile_queues.write().await;
        
        pile_queues.insert(pile.number.clone(), VecDeque::with_capacity(PILE_QUEUE_CAPACITY));
        piles.insert(pile.number.clone(), pile);
    }

    // 添加充电请求到等候区
    pub async fn add_to_waiting_queue(&self, request: Arc<ChargingRequest>) -> Result<(), String> {
        let mut queue = self.waiting_queue.write().await;
        
        if queue.len() >= WAITING_AREA_CAPACITY {
            return Err("等候区已满".to_string());
        }
        
        queue.push_back(request);
        Ok(())
    }

    // 从等候区移除请求
    pub async fn remove_from_waiting_queue(&self, request_id: Uuid) -> Option<Arc<ChargingRequest>> {
        let mut queue = self.waiting_queue.write().await;
        let position = queue.iter().position(|r| r.id == request_id);
        
        if let Some(pos) = position {
            queue.remove(pos)
        } else {
            None
        }
    }

    // 获取充电桩队列长度
    pub async fn get_pile_queue_length(&self, pile_id: &str) -> usize {
        let pile_queues = self.pile_queues.read().await;
        pile_queues.get(pile_id).map(|q| q.len()).unwrap_or(0)
    }

    // 获取等候区中指定模式的等待数量
    pub async fn get_waiting_count(&self, mode: ChargingMode) -> usize {
        let queue = self.waiting_queue.read().await;
        queue.iter().filter(|r| r.mode == mode).count()
    }

    // 获取可用的充电桩
    pub async fn get_available_piles(&self, mode: ChargingMode) -> Vec<Arc<ChargingPile>> {
        let piles = self.piles.read().await;
        let pile_queues = self.pile_queues.read().await;
        
        piles.values()
            .filter(|p| {
                p.mode == mode 
                && p.status == PileStatus::Available
                && pile_queues.get(&p.number)
                    .map(|q| q.len() < PILE_QUEUE_CAPACITY)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    // 将请求添加到充电桩队列
    pub async fn add_to_pile_queue(
        &self,
        pile_id: &str,
        request: Arc<ChargingRequest>,
    ) -> Result<(), String> {
        let mut pile_queues = self.pile_queues.write().await;
        
        if let Some(queue) = pile_queues.get_mut(pile_id) {
            if queue.len() >= PILE_QUEUE_CAPACITY {
                return Err("充电桩队列已满".to_string());
            }
            queue.push_back(request);
            Ok(())
        } else {
            Err("充电桩不存在".to_string())
        }
    }

    // 从充电桩队列移除请求
    pub async fn remove_from_pile_queue(
        &self,
        pile_id: &str,
        request_id: Uuid,
    ) -> Option<Arc<ChargingRequest>> {
        let mut pile_queues = self.pile_queues.write().await;
        
        if let Some(queue) = pile_queues.get_mut(pile_id) {
            let position = queue.iter().position(|r| r.id == request_id);
            if let Some(pos) = position {
                queue.remove(pos)
            } else {
                None
            }
        } else {
            None
        }
    }

    // 获取等候区队列
    pub async fn get_waiting_queue(&self) -> Vec<Arc<ChargingRequest>> {
        let queue = self.waiting_queue.read().await;
        queue.iter().cloned().collect()
    }

    // 获取所有充电桩
    pub async fn get_piles(&self) -> HashMap<String, Arc<ChargingPile>> {
        self.piles.read().await.clone()
    }

    // 获取指定充电桩的队列
    pub async fn get_pile_queue(&self, pile_id: &str) -> Option<Vec<Arc<ChargingRequest>>> {
        let pile_queues = self.pile_queues.read().await;
        pile_queues.get(pile_id).map(|q| q.iter().cloned().collect())
    }

    // 清空指定充电桩的队列
    pub async fn clear_pile_queue(&self, pile_id: &str) -> Result<(), String> {
        let mut pile_queues = self.pile_queues.write().await;
        if let Some(queue) = pile_queues.get_mut(pile_id) {
            queue.clear();
            Ok(())
        } else {
            Err("充电桩不存在".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChargingRequest;

    #[tokio::test]
    async fn test_waiting_queue_operations() {
        let manager = QueueManager::new();
        let request = Arc::new(ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        ));

        // 测试添加到等候区
        assert!(manager.add_to_waiting_queue(request.clone()).await.is_ok());
        
        // 测试获取等待数量
        assert_eq!(manager.get_waiting_count(ChargingMode::Fast).await, 1);
        
        // 测试移除请求
        let removed = manager.remove_from_waiting_queue(request.id).await;
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, request.id);
    }

    #[tokio::test]
    async fn test_pile_queue_operations() {
        let manager = QueueManager::new();
        let pile = Arc::new(ChargingPile::new("A1".to_string(), ChargingMode::Fast));
        
        // 添加充电桩
        manager.add_pile(pile.clone()).await;
        
        let request = Arc::new(ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        ));

        // 测试添加到充电桩队列
        assert!(manager.add_to_pile_queue(&pile.number, request.clone()).await.is_ok());
        
        // 测试获取队列长度
        assert_eq!(manager.get_pile_queue_length(&pile.number).await, 1);
        
        // 测试移除请求
        let removed = manager.remove_from_pile_queue(&pile.number, request.id).await;
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, request.id);
    }
} 