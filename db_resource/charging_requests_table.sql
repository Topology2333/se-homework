-- 充电请求表
DROP TABLE IF EXISTS `charging_requests`;
CREATE TABLE `charging_requests` (
  `id` binary(16) NOT NULL,
  `user_id` binary(16) NOT NULL,
  `mode` enum('Fast','Slow') NOT NULL,
  `amount` double NOT NULL,
  `queue_number` varchar(10) NOT NULL,
  `status` enum('Waiting','Charging','Completed','Cancelled') NOT NULL DEFAULT 'Waiting',
  `created_at` datetime(6) NOT NULL,
  `updated_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
  PRIMARY KEY (`id`),
  KEY `user_id` (`user_id`),
  KEY `status` (`status`),
  KEY `mode` (`mode`),
  KEY `queue_number` (`queue_number`),
  FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci; 