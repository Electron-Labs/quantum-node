#!/bin/bash

# MySQL/MariaDB connection parameters
DB_USER="root"
DB_PASS="temp123"
DB_NAME="quantum"

# Create the database and tables
mysql -u"$DB_USER" -p"$DB_PASS" -e "
CREATE DATABASE IF NOT EXISTS $DB_NAME;

USE $DB_NAME;

CREATE TABLE IF NOT EXISTS reduction_circuit (
  circuit_id VARCHAR(255) PRIMARY KEY,
  proving_key_path VARCHAR(255),
  vk_path VARCHAR(255),
  pis_len INT,
  proving_scheme VARCHAR(255),
  KEY idx_pis_len (pis_len)
);

CREATE TABLE IF NOT EXISTS protocol (
  protocol_name varchar(255),
  auth_token varchar(255) DEFAULT NULL,
  PRIMARY KEY (protocol_name)
);

CREATE TABLE IF NOT EXISTS auth (
  auth_token varchar(255),
  is_master INT DEFAULT 0,
  PRIMARY KEY (auth_token)
);

CREATE TABLE IF NOT EXISTS user_circuit_data (
  circuit_hash VARCHAR(255) PRIMARY KEY,
  vk_path VARCHAR(255),
  reduction_circuit_id VARCHAR(255) DEFAULT NULL,
  pis_len INT,
  proving_scheme VARCHAR(255),
  circuit_reduction_status INT,
  protocol_name VARCHAR(255),
  FOREIGN KEY (reduction_circuit_id) REFERENCES reduction_circuit(circuit_id),
  FOREIGN KEY (protocol_name) REFERENCES protocol(protocol_name)
);

CREATE TABLE IF NOT EXISTS task (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user_circuit_hash VARCHAR(255),
  task_type INT,
  proof_id VARCHAR(255),
  task_status INT
);

CREATE INDEX id_task_status ON task(task_status);


CREATE TABLE IF NOT EXISTS proof (
  id INT AUTO_INCREMENT PRIMARY KEY,
  proof_hash VARCHAR(255),
  pis_path VARCHAR(255),
  proof_path VARCHAR(255),
  reduction_proof_path VARCHAR(255),
  reduction_proof_pis_path VARCHAR(255),
  superproof_id INT,
  reduction_time INT,
  proof_status INT,
  user_circuit_hash VARCHAR(255),
  public_inputs varchar(1200) DEFAULT NULL,
  FOREIGN KEY (user_circuit_hash) REFERENCES user_circuit_data(circuit_hash)
);

CREATE INDEX idx_proof_status ON proof(proof_status);

CREATE TABLE IF NOT EXISTS superproof (
  id INT AUTO_INCREMENT PRIMARY KEY,
  proof_ids VARCHAR(255),
  superproof_proof_path VARCHAR(255),
  superproof_pis_path VARCHAR(255),
  transaction_hash VARCHAR(255),
  gas_cost DECIMAL(18,3) DEFAULT NULL,
  eth_price DECIMAL(18,3) DEFAULT NULL,
  agg_time INT,
  status INT,
  superproof_root VARCHAR(255),
  superproof_leaves_path VARCHAR(255),
  onchain_submission_time datetime DEFAULT NULL,
  total_proof_ver_cost decimal(18,2) DEFAULT NULL,
  total_cost_usd decimal(18,2) DEFAULT NULL,
  total_proving_time decimal(18,2) DEFAULT NULL,
  previous_superproof_root varchar(255) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS proof_submission_config (
  proof_submission_time int DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS proof_system (
  proof_system varchar(255) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS protocol_ui (
  protocol_name varchar(255) DEFAULT NULL,
  is_aggregated int DEFAULT '0',
  display_name varchar(255) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS superproof_contract_config (
  address varchar(255) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS cost_saved (
  total_gas_saved DECIMAL(18,2) DEFAULT 0,
  total_usd_saved DECIMAL(18,2) DEFAULT 0
);

INSERT INTO cost_saved VALUES (0,0);

"

