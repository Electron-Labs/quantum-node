SET @DBNAME = 'quantum';

SET @SQL = CONCAT('CREATE DATABASE IF NOT EXISTS ', @DBNAME);
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;


SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.reduction_circuit (
circuit_id VARCHAR(255) PRIMARY KEY, 
proving_key_path VARCHAR(255),  
vk_path VARCHAR(255),  
pis_len INT, 
proving_scheme VARCHAR(255),
KEY idx_pis_len (pis_len)
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.protocol (
  protocol_name varchar(255),
  auth_token varchar(255) DEFAULT NULL,
  PRIMARY KEY (protocol_name)
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;


SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.auth (
  auth_token varchar(255),
  is_master INT DEFAULT 0,
  PRIMARY KEY (auth_token)
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.user_circuit_data (
  circuit_hash VARCHAR(255) PRIMARY KEY,
  vk_path VARCHAR(255),
  reduction_circuit_id VARCHAR(255) DEFAULT NULL,
  pis_len INT,
  proving_scheme VARCHAR(255),
  circuit_reduction_status INT,
  protocol_name VARCHAR(255),
  FOREIGN KEY (reduction_circuit_id) REFERENCES reduction_circuit(circuit_id),
  FOREIGN KEY (protocol_name) REFERENCES protocol(protocol_name)
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.task (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user_circuit_hash VARCHAR(255),
  task_type INT,
  proof_id VARCHAR(255),
  task_status INT
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @SQL = CONCAT('CREATE INDEX id_task_status ON ', @DBNAME,'.task(task_status)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;


SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.proof (
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
  FOREIGN KEY (user_circuit_hash) REFERENCES user_circuit_data(circuit_hash)
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;


SET @SQL = CONCAT('CREATE INDEX id_task_status ON ', @DBNAME,'.proof(proof_status)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;


SET @SQL = CONCAT('CREATE TABLE IF NOT EXISTS ', @DBNAME,'.superproof (
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
  onchain_submission_time datetime DEFAULT NULL
)');
PREPARE stmt FROM @SQL;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;