# auria-config

Configuration management for AURIA Runtime Core.

## Overview

Handles loading, validation, and management of runtime configuration from TOML files and environment variables.

## Configuration Structure

```toml
[node]
id = "node-001"
wallet_address = "0x..."
data_dir = "./data"
log_level = "info"

[storage]
cache_dir = "./cache"
max_cache_size_gb = 10
network_storage_enabled = true

[network]
http_port = 8080
grpc_port = 50051
p2p_enabled = true
p2p_port = 30303

[execution]
enabled_tiers = ["nano", "standard"]
gpu_enabled = true
max_batch_size = 8

[settlement]
settlement_enabled = true
settlement_interval_seconds = 3600
```

## Usage

```rust
use auria_config::{load_config, save_config, Config};

let config = load_config(&path_to_config)?;
save_config(&config, &path)?;
```
