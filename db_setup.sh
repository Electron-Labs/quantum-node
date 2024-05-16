#!/bin/bash

# MySQL/MariaDB connection parameters
DB_USER="root"
DB_PASS="sql123"
DB_HOST="localhost"
DB_NAME="quantum"

# Create the database and tables
mysql -u root -p -e "
CREATE DATABASE IF NOT EXISTS quantum;

USE quantum;

CREATE TABLE IF NOT EXISTS user_circuit_data (
  circuit_hash VARCHAR(255) PRIMARY KEY,
  vk_path VARCHAR(255),
  cd_path VARCHAR(255),
  reduction_circuit_id INT DEFAULT NULL,
  pis_len INT                   
);

CREATE TABLE IF NOT EXISTS reduction_circuit (
  id INT AUTO_INCREMENT PRIMARY KEY,
  proving_key_path VARCHAR(255),
  vk_path VARCHAR(255),
  pis_len INT,
  KEY idx_pis_len (pis_len)
);

CREATE TABLE IF NOT EXISTS tasks (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user_circuit_hash VARCHAR(255),
  task_type INT,
  proof_id INTc,
  proof_status INT,
  ciruit_reduction_status INT
);

CREATE INDEX id_circuit_reduction_status ON tasks(ciruit_reduction_status);
CREATE INDEX idx_proof_status ON tasks(proof_status);

CREATE TABLE IF NOT EXISTS proofs (
  id INT AUTO_INCREMENT PRIMARY KEY,
  proof_hash VARCHAR(255),
  pis_path VARCHAR(255),
  proof_path VARCHAR(255),
  reduction_proof_path VARCHAR(255),
  reduction_proof_pis_path VARCHAR(255),
  superproof_id INT,
  reduction_time INT
);

CREATE TABLE IF NOT EXISTS superproofs (
  id INT AUTO_INCREMENT PRIMARY KEY,
  proof_ids VARCHAR(255),
  superproof_proof_path VARCHAR(255),
  superproof_pis_path VARCHAR(255),
  transaction_hash VARCHAR(255),
  gas_cost DECIMAL(18,3) DEFAULT NULL,
  agg_time INT
);
"