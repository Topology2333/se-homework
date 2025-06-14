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



SET FOREIGN_KEY_CHECKS = 0;
INSERT INTO charging_requests
        (id, user_id, mode, amount, queue_number, status, created_at)
VALUES
-- F1 ------------------------------------------------------------------------
(UNHEX(REPLACE('11111111-1111-4111-8111-111111111111','-','')),
 UNHEX(REPLACE('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa','-','')),
 'Fast',  30.00, 'F1', 'waiting',  NOW()),
(UNHEX(REPLACE('22222222-2222-4222-8222-222222222222','-','')),
 UNHEX(REPLACE('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb','-','')),
 'Fast',  35.50, 'F1', 'charging', NOW()),

-- F2 ------------------------------------------------------------------------
(UNHEX(REPLACE('33333333-3333-4333-8333-333333333333','-','')),
 UNHEX(REPLACE('cccccccc-cccc-4ccc-8ccc-cccccccccccc','-','')),
 'Fast',  45.25, 'F2', 'completed', NOW()),
(UNHEX(REPLACE('44444444-4444-4444-8444-444444444444','-','')),
 UNHEX(REPLACE('dddddddd-dddd-4ddd-8ddd-dddddddddddd','-','')),
 'Fast',  40.00, 'F2', 'cancelled', NOW()),

-- S1 ------------------------------------------------------------------------
(UNHEX(REPLACE('55555555-5555-4555-8555-555555555555','-','')),
 UNHEX(REPLACE('eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee','-','')),
 'Slow',  20.00, 'S1', 'waiting',  NOW()),

-- S2 ------------------------------------------------------------------------
(UNHEX(REPLACE('66666666-6666-4666-8666-666666666666','-','')),
 UNHEX(REPLACE('ffffffff-ffff-4fff-8fff-ffffffffffff','-','')),
 'Slow',  22.00, 'S2', 'charging', NOW()),

-- S3 ------------------------------------------------------------------------
(UNHEX(REPLACE('77777777-7777-4777-8777-777777777777','-','')),
 UNHEX(REPLACE('99999999-9999-4999-8999-999999999999','-','')),
 'Slow',  18.75, 'S3', 'waiting',  NOW());

SET FOREIGN_KEY_CHECKS = 1;