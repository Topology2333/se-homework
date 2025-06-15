-- 创建充电桩表
CREATE TABLE charging_piles (
    id BINARY(16) PRIMARY KEY,
    number VARCHAR(20) NOT NULL,
    mode ENUM('Fast', 'Slow') NOT NULL,
    status ENUM('Available', 'Charging', 'Shutdown', 'Fault') NOT NULL,
    total_charge_count INT NOT NULL,
    total_charge_time DOUBLE NOT NULL,
    total_charge_amount DOUBLE NOT NULL,
    total_charging_fee DOUBLE NOT NULL,
    total_service_fee DOUBLE NOT NULL,
    started_at DATETIME NULL
);

INSERT INTO charging_piles (id, number, mode, status, total_charge_count, total_charge_time, total_charge_amount, total_charging_fee, total_service_fee, started_at) 
VALUES
(UUID_TO_BIN(UUID()), 'F1', 'Fast', 'Available', 0, 0.0, 0.0, 0.0, 0.0, NULL),
(UUID_TO_BIN(UUID()), 'F2', 'Fast', 'Available', 0, 0.0, 0.0, 0.0, 0.0, NULL),
(UUID_TO_BIN(UUID()), 'T1', 'Slow', 'Available', 0, 0.0, 0.0, 0.0, 0.0, NULL),
(UUID_TO_BIN(UUID()), 'T2', 'Slow', 'Available', 0, 0.0, 0.0, 0.0, 0.0, NULL),
(UUID_TO_BIN(UUID()), 'T3', 'Slow', 'Available', 0, 0.0, 0.0, 0.0, 0.0, NULL);