//!
//! # 基本类型定义
//!

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// VM CPU 默认数量
pub const CPU_DEFAULT: u32 = 2;
/// VM MEM 默认容量, 单位: MB
pub const MEM_DEFAULT: u32 = 2 * 1024;
/// VM DISK 默认容量, 单位: MB
pub const DISK_DEFAULT: u32 = 16 * 1024;

/// Cli ID
pub type CliId = String;
/// Cli ID as `&str`
pub type CliIdRef = str;
/// Env ID
pub type EnvId = String;
/// Env ID as `&str`
pub type EnvIdRef = str;

/// 使用 Vm 的 MAC 地址的末尾两段的乘积, 最大值: 256 * 256
pub type VmId = i32;
pub type Pid = u32;

/// eg: 10.10.123.110
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Ipv4 {
    addr: String,
}

impl Ipv4 {
    /// create a new one
    pub fn new(addr: String) -> Ipv4 {
        Ipv4 { addr }
    }

    /// convert to string
    pub fn as_str(&self) -> &str {
        self.addr.as_str()
    }
}

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.addr)
    }
}

/// eg: 22
pub type Port = u16;
/// Vm 内部视角的端口, 如 80、443 等标准端口
pub type VmPort = Port;
/// 外部视角的端口, 如 8080、8443 等 nat 出来的端口
pub type PubPort = Port;

/// 未来可能支持更多的容器引擎
/// - [Y] Bhyve
/// - [Y] Qemu
/// - [N] Jail
/// - [N] Docker
/// - [N] Systemd Nspawn
/// - [N] Firecracker
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum VmKind {
    /// Bhyve 虚拟机
    Bhyve,
    /// Qemu 虚拟机
    Qemu,
    /// Jail 容器
    Jail,
    /// Docker 容器
    Docker,
    /// 未知项
    Unknown,
}

impl fmt::Display for VmKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmKind::Bhyve => write!(f, "Bhyve"),
            VmKind::Qemu => write!(f, "Qemu"),
            VmKind::Jail => write!(f, "Jail"),
            VmKind::Docker => write!(f, "Docker"),
            _ => write!(f, "Unknown"),
        }
    }
}

impl Default for VmKind {
    #[cfg(target_os = "linux")]
    fn default() -> VmKind {
        VmKind::Qemu
    }
    #[cfg(target_os = "freebsd")]
    fn default() -> VmKind {
        VmKind::Bhyve
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    fn default() -> VmKind {
        VmKind::Unknown
    }
}

/// 元信息, 用于展示
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnvMeta {
    /// 保证全局唯一
    pub id: EnvId,
    /// 起始时间设定之后不允许变更
    pub start_timestamp: u64,
    /// 结束时间可以变更, 用以控制 Vm 的生命周期
    pub end_timestamp: u64,
    /// 内部的 Vm 数量
    pub vm_cnt: usize,
}

/// 环境实例的详细信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnvInfo {
    /// 保证全局唯一
    pub id: EnvId,
    /// 起始时间设定之后不允许变更
    pub start_timestamp: u64,
    /// 结束时间可以变更, 用以控制 Vm 的生命周期
    pub end_timestamp: u64,
    /// 同一 Env 下所有 Vm 集合
    pub vm: HashMap<VmId, VmInfo>,
}

/// 以此结构响应客户端请求, 防止触发 Drop 动作
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmInfo {
    /// 系统名称
    pub os: String,
    /// 虚拟实例的类型
    pub kind: VmKind,
    /// CPU 数量
    pub cpu_num: u32,
    /// 单位: MB
    pub mem_size: u32,
    /// 单位: MB
    pub disk_size: u32,
    /// Vm IP 由 VmId 决定, 使用'10.10.x.x/8'网段
    pub ip: Ipv4,
    /// 用于 DNAT 的内外端口影射关系,
    pub port_map: HashMap<VmPort, PubPort>,
}
