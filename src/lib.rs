// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Configuration management for AURIA Runtime Core.
//     Handles loading, validation, and management of runtime configuration
//     from TOML files and environment variables.
//
use auria_core::{AuriaError, AuriaResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod hardware;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub execution: ExecutionConfig,
    pub settlement: SettlementConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub wallet_address: Option<String>,
    pub data_dir: PathBuf,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub cache_dir: PathBuf,
    pub max_cache_size_gb: u64,
    pub network_storage_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub http_port: u16,
    pub grpc_port: u16,
    pub p2p_enabled: bool,
    pub p2p_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub enabled_tiers: Vec<String>,
    pub gpu_enabled: bool,
    pub max_batch_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    pub settlement_enabled: bool,
    pub settlement_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node: NodeConfig {
                id: uuid::Uuid::new_v4().to_string(),
                wallet_address: None,
                data_dir: PathBuf::from("./data"),
                log_level: "info".to_string(),
            },
            storage: StorageConfig {
                cache_dir: PathBuf::from("./cache"),
                max_cache_size_gb: 10,
                network_storage_enabled: true,
            },
            network: NetworkConfig {
                http_port: 8080,
                grpc_port: 50051,
                p2p_enabled: false,
                p2p_port: 30303,
            },
            execution: ExecutionConfig {
                enabled_tiers: vec!["nano".to_string()],
                gpu_enabled: true,
                max_batch_size: 8,
            },
            settlement: SettlementConfig {
                settlement_enabled: true,
                settlement_interval_seconds: 3600,
            },
        }
    }
}

pub fn load_config(path: &PathBuf) -> AuriaResult<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| AuriaError::ConfigError(format!("Failed to read config: {}", e)))?;
    let config: Config = toml::from_str(&content)
        .map_err(|e| AuriaError::ConfigError(format!("Failed to parse config: {}", e)))?;
    Ok(config)
}

pub fn save_config(config: &Config, path: &PathBuf) -> AuriaResult<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| AuriaError::ConfigError(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(path, content)
        .map_err(|e| AuriaError::ConfigError(format!("Failed to write config: {}", e)))?;
    Ok(())
}
