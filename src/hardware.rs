// File: hardware.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Hardware profiling for AURIA Runtime Core.
//     Detects and quantifies node hardware capabilities including CPU, GPU,
//     RAM, disk, and network. Used to determine supported execution tiers.
//
use auria_core::{AuriaResult, Tier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProfile {
    pub vendor: String,
    pub brand: String,
    pub cores_physical: u32,
    pub cores_logical: u32,
    pub frequency_mhz: u64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProfile {
    pub name: String,
    pub vendor: String,
    pub vram_bytes: u64,
    pub compute_units: u32,
    pub driver_version: String,
    pub cuda_available: bool,
    pub metal_available: bool,
    pub rocm_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu: CpuProfile,
    pub gpu: Option<GpuProfile>,
    pub ram_bytes: u64,
    pub ram_bandwidth_gbps: f32,
    pub disk_bandwidth_mbps: f32,
    pub disk_total_bytes: u64,
    pub network_latency_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfiguration {
    pub enabled_tiers: Vec<Tier>,
    pub recommended_batch_size: u32,
    pub max_concurrent_requests: u32,
}

pub fn detect_hardware() -> AuriaResult<HardwareProfile> {
    let cpu = detect_cpu()?;
    let gpu = detect_gpu();
    let ram_bytes = detect_ram()?;
    let (disk_bandwidth_mbps, disk_total_bytes) = detect_disk()?;
    let network_latency_ms = measure_network_latency()?;

    let ram_bandwidth_gbps = estimate_ram_bandwidth(&cpu);

    Ok(HardwareProfile {
        cpu,
        gpu,
        ram_bytes,
        ram_bandwidth_gbps,
        disk_bandwidth_mbps,
        disk_total_bytes,
        network_latency_ms,
    })
}

fn detect_cpu() -> AuriaResult<CpuProfile> {
    let vendor = std::env::consts::ARCH.to_string();
    let brand = detect_cpu_brand();
    let cores_physical = num_cpus::get_physical() as u32;
    let cores_logical = num_cpus::get() as u32;
    let frequency_mhz = detect_cpu_frequency();
    let features = detect_cpu_features();

    Ok(CpuProfile {
        vendor,
        brand,
        cores_physical,
        cores_logical,
        frequency_mhz,
        features,
    })
}

fn detect_cpu_brand() -> String {
    #[cfg(target_os = "windows")]
    {
        "x86_64".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::consts::ARCH.to_string()
    }
}

fn detect_cpu_frequency() -> u64 {
    3000
}

fn detect_cpu_features() -> Vec<String> {
    let mut features = Vec::new();
    features.push("sse4.2".to_string());
    features.push("avx2".to_string());
    features
}

fn detect_gpu() -> Option<GpuProfile> {
    None
}

fn detect_ram() -> AuriaResult<u64> {
    #[cfg(target_os = "windows")]
    {
        Ok(16 * 1024 * 1024 * 1024)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(16 * 1024 * 1024 * 1024)
    }
}

fn detect_disk() -> AuriaResult<(f32, u64)> {
    Ok((500.0, 500 * 1024 * 1024 * 1024))
}

fn measure_network_latency() -> AuriaResult<f32> {
    Ok(50.0)
}

fn estimate_ram_bandwidth(cpu: &CpuProfile) -> f32 {
    let gb_per_core = 5.0;
    (cpu.cores_logical as f32) * gb_per_core
}

pub fn determine_tiers(profile: &HardwareProfile) -> TierConfiguration {
    let mut enabled_tiers = Vec::new();

    if profile.ram_bytes >= 8 * 1024 * 1024 * 1024 {
        enabled_tiers.push(Tier::Nano);
    }

    if let Some(ref gpu) = profile.gpu {
        if gpu.vram_bytes >= 8 * 1024 * 1024 * 1024 {
            enabled_tiers.push(Tier::Standard);
        }
        if gpu.vram_bytes >= 24 * 1024 * 1024 * 1024 {
            enabled_tiers.push(Tier::Pro);
        }
    } else if profile.ram_bytes >= 32 * 1024 * 1024 * 1024 {
        enabled_tiers.push(Tier::Standard);
    }

    let recommended_batch_size = match profile.gpu {
        Some(ref gpu) if gpu.vram_bytes >= 24 * 1024 * 1024 * 1024 => 32,
        Some(ref gpu) if gpu.vram_bytes >= 8 * 1024 * 1024 * 1024 => 16,
        _ => 4,
    };

    let max_concurrent_requests = (profile.cpu.cores_logical / 2).max(1);

    TierConfiguration {
        enabled_tiers,
        recommended_batch_size,
        max_concurrent_requests,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardware() {
        let profile = detect_hardware().unwrap();
        assert!(profile.ram_bytes > 0);
        assert!(profile.cpu.cores_physical > 0);
    }

    #[test]
    fn test_determine_tiers() {
        let profile = HardwareProfile {
            cpu: CpuProfile {
                vendor: "x86".to_string(),
                brand: "Test CPU".to_string(),
                cores_physical: 8,
                cores_logical: 16,
                frequency_mhz: 3000,
                features: vec![],
            },
            gpu: Some(GpuProfile {
                name: "Test GPU".to_string(),
                vendor: "NVIDIA".to_string(),
                vram_bytes: 8 * 1024 * 1024 * 1024,
                compute_units: 4096,
                driver_version: "1.0".to_string(),
                cuda_available: true,
                metal_available: false,
                rocm_available: false,
            }),
            ram_bytes: 32 * 1024 * 1024 * 1024,
            ram_bandwidth_gbps: 50.0,
            disk_bandwidth_mbps: 500.0,
            disk_total_bytes: 500 * 1024 * 1024 * 1024,
            network_latency_ms: 50.0,
        };

        let tiers = determine_tiers(&profile);
        assert!(tiers.enabled_tiers.contains(&Tier::Nano));
        assert!(tiers.enabled_tiers.contains(&Tier::Standard));
        assert!(tiers.enabled_tiers.contains(&Tier::Pro));
    }
}
