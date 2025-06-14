-- 创建充电桩表
CREATE TABLE IF NOT EXISTS `charging_piles` (
    `id` BINARY(16) NOT NULL,
    `number` VARCHAR(255) NOT NULL UNIQUE,
    `mode` ENUM('Fast', 'Slow') NOT NULL,
    `status` ENUM('Available', 'Charging', 'Shutdown', 'Fault') NOT NULL DEFAULT 'Available',
    `total_charge_count` INT NOT NULL DEFAULT 0,
    `total_charge_time` DOUBLE NOT NULL DEFAULT 0.0,
    `total_charge_amount` DOUBLE NOT NULL DEFAULT 0.0,
    `total_charging_fee` DOUBLE NOT NULL DEFAULT 0.0,
    `total_service_fee` DOUBLE NOT NULL DEFAULT 0.0,
    `started_at` DATETIME NULL,
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    INDEX `idx_number` (`number`),
    INDEX `idx_status` (`status`),
    INDEX `idx_mode` (`mode`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 插入一些示例数据
INSERT INTO `charging_piles` (`id`, `number`, `mode`, `status`) VALUES 
(UUID_TO_BIN(UUID()), 'CP001', 'Fast', 'Available'),
(UUID_TO_BIN(UUID()), 'CP002', 'Fast', 'Available'),
(UUID_TO_BIN(UUID()), 'CP003', 'Slow', 'Available'),
(UUID_TO_BIN(UUID()), 'CP004', 'Slow', 'Available'),
(UUID_TO_BIN(UUID()), 'CP005', 'Fast', 'Available'); 