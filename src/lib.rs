pub mod models;
pub mod scheduler;
pub mod billing;

use std::sync::atomic::{AtomicUsize, Ordering};

// 全局计数器，初始值为3
pub static QUEUE_COUNTER: AtomicUsize = AtomicUsize::new(3);

// 获取并递增队列号码的函数
pub fn get_next_queue_number() -> usize {
    QUEUE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub use models::*;
pub use scheduler::*; 