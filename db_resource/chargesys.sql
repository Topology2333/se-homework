-- MySQL dump 10.13  Distrib 8.0.42, for Win64 (x86_64)
--
-- Host: localhost    Database: chargesys
-- ------------------------------------------------------
-- Server version	8.0.42

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `charging_records`
--

DROP TABLE IF EXISTS `charging_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `charging_records` (
  `id` binary(16) NOT NULL,
  `user_id` binary(16) NOT NULL,
  `pile_id` varchar(255) NOT NULL,
  `mode` enum('Fast','Slow') NOT NULL,
  `charging_amount` double NOT NULL,
  `charging_fee` double NOT NULL,
  `service_fee` double NOT NULL,
  `total_fee` double NOT NULL,
  `start_time` datetime NOT NULL,
  `end_time` datetime NOT NULL,
  `created_at` datetime NOT NULL,
  `charging_time` double GENERATED ALWAYS AS ((timestampdiff(SECOND,`start_time`,`end_time`) / 3600.0)) STORED,
  PRIMARY KEY (`id`),
  KEY `user_id` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `charging_records`
--

LOCK TABLES `charging_records` WRITE;
/*!40000 ALTER TABLE `charging_records` DISABLE KEYS */;
INSERT INTO `charging_records` (`id`, `user_id`, `pile_id`, `mode`, `charging_amount`, `charging_fee`, `service_fee`, `total_fee`, `start_time`, `end_time`, `created_at`) VALUES (_binary 'Š6©DO\ð±ø\0]ý™\0',_binary 'y\Ø\ÞLÅ”R\ìÔª\Ò.\Ð','CHARGE_PILE_XYZ','Fast',15.5,10,2,12,'2025-06-07 10:00:00','2025-06-07 11:12:00','2025-06-08 17:57:38'),(_binary 'V+küD]\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CHARGE_PILE_12','Fast',15.5,10,2,12,'2025-06-07 10:00:00','2025-06-07 11:12:00','2025-06-08 19:40:11'),(_binary 'Y\éÝ¡Dd\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CHARGE_PILE_12','Fast',15.5,10,2,12,'2025-06-07 12:00:00','2025-06-07 14:12:00','2025-06-08 20:30:24'),(_binary '\áiDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP001','Fast',25.5,20,5,25,'2024-05-01 08:00:00','2024-05-01 09:30:00','2025-06-08 20:41:21'),(_binary '\áohDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP002','Slow',12.3,10,3,13,'2024-05-02 14:30:00','2024-05-02 17:42:00','2025-06-08 20:41:21'),(_binary '\árp^De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP003','Fast',30.1,25,6,31,'2024-05-03 10:15:00','2024-05-03 12:03:00','2025-06-08 20:41:21'),(_binary '\áv\ÊDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP001','Slow',8.9,7,2.5,9.5,'2024-05-04 19:00:00','2024-05-04 21:30:00','2025-06-08 20:41:21'),(_binary '\áz\ó\ÉDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP004','Fast',20,16,4,20,'2024-05-05 09:45:00','2024-05-05 11:00:00','2025-06-08 20:41:21'),(_binary '\á£7De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP002','Slow',15,12,3.5,15.5,'2024-05-06 13:00:00','2024-05-06 17:00:00','2025-06-08 20:41:21'),(_binary '\á‚\ËDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP005','Fast',28.7,23,5.5,28.5,'2024-05-07 11:30:00','2024-05-07 13:06:00','2025-06-08 20:41:21'),(_binary '\á†g–De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP003','Slow',10.5,8.5,2,10.5,'2024-05-08 16:00:00','2024-05-08 18:48:00','2025-06-08 20:41:21'),(_binary '\á‰Y\ÔDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP001','Fast',22,18,4.5,22.5,'2024-05-09 07:30:00','2024-05-09 08:48:00','2025-06-08 20:41:21'),(_binary '\á\ÕDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP004','Slow',18.2,14.5,4,18.5,'2024-05-10 20:00:00','2024-05-10 23:42:00','2025-06-08 20:41:21'),(_binary 'á‘‡’De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP006','Fast',27,21.5,5,26.5,'2024-05-11 09:00:00','2024-05-11 10:24:00','2025-06-08 20:41:21'),(_binary '\á•üDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP002','Slow',9.8,7.8,2.2,10,'2024-05-12 15:30:00','2024-05-12 17:36:00','2025-06-08 20:41:21'),(_binary '\á™T­De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP007','Fast',33.5,28,6.5,34.5,'2024-05-13 10:00:00','2024-05-13 11:54:00','2025-06-08 20:41:21'),(_binary 'áž¢\æDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP003','Slow',14.1,11,3,14,'2024-05-14 18:45:00','2024-05-14 21:45:00','2025-06-08 20:41:21'),(_binary '\á¢\ÙDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP001','Fast',29,24,5.8,29.8,'2024-05-15 08:30:00','2024-05-15 10:12:00','2025-06-08 20:41:21'),(_binary '\á¨/—De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP005','Slow',11.5,9,2.5,11.5,'2024-05-16 14:00:00','2024-05-16 16:36:00','2025-06-08 20:41:21'),(_binary '\á¬þ8De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP008','Fast',26.3,21,5.3,26.3,'2024-05-17 11:00:00','2024-05-17 12:30:00','2025-06-08 20:41:21'),(_binary '\á±>BDe\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP004','Slow',13,10.5,2.8,13.3,'2024-05-18 17:15:00','2024-05-18 20:21:00','2025-06-08 20:41:21'),(_binary '\áµ •De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP001','Fast',31.8,26.5,6,32.5,'2024-05-19 09:10:00','2024-05-19 10:58:00','2025-06-08 20:41:21'),(_binary '\â:}»De\ð±ø\0]ý™\0',_binary '’ Q²Aÿ¾.#X\Æ\æÁ','CP009','Slow',16.7,13.2,3.5,16.7,'2024-05-20 16:30:00','2024-05-20 20:00:00','2025-06-08 20:41:22');
/*!40000 ALTER TABLE `charging_records` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `users`
--

DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` binary(16) NOT NULL,
  `username` varchar(255) NOT NULL,
  `password_hash` varchar(255) NOT NULL,
  `is_admin` tinyint(1) NOT NULL DEFAULT '0',
  `created_at` datetime(6) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `users`
--

LOCK TABLES `users` WRITE;
/*!40000 ALTER TABLE `users` DISABLE KEYS */;
INSERT INTO `users` VALUES (_binary 'y\Ø\ÞLÅ”R\ìÔª\Ò.\Ð','Hunger','dd13c90b6042bd26042b0ac42ec21dfe0269e71c0b26eb7ac725c91d0fe1836c',0,'2025-06-08 06:18:22.146289'),(_binary '’ Q²Aÿ¾.#X\Æ\æÁ','Hunger2','dd13c90b6042bd26042b0ac42ec21dfe0269e71c0b26eb7ac725c91d0fe1836c',0,'2025-06-08 11:32:10.439804');
/*!40000 ALTER TABLE `users` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2025-06-08 20:53:16
