//!
//! # Virtual Machine Mgmt
//!
//! - Qemu
//! - Docker
//! - ...
//!

pub(in crate::linux) mod cgroup;
pub(in crate::linux) mod engine;
pub(in crate::linux) mod util;

use crate::Vm;
use myutil::err::*;
#[cfg(not(feature = "testmock"))]
use myutil::*;
#[cfg(not(feature = "testmock"))]
#[cfg(feature = "backing_file")]
use std::fs;
#[cfg(not(feature = "testmock"))]
use std::process;

#[inline(always)]
#[cfg(feature = "testmock")]
pub(crate) fn start(_: &Vm) -> Result<()> {
    Ok(())
}

#[inline(always)]
#[cfg(not(feature = "testmock"))]
pub(crate) fn start(vm: &Vm) -> Result<()> {
    // 1. 首先, 分配 Cgroup 挂载点
    // 2. 之后, 控制进制先进入 Vm 的 Cgroup
    //   - 其创建的 Vm 进程会自动归属于相同的 Cgroup
    //   - 在清理 Vm 进程时要跳过可能存在的控制进程 PID
    cgroup::alloc_mnt_point(vm.id)
        .c(d!())
        .and_then(|_| cgroup::add_vm(vm.id, process::id()).c(d!()))
        .and_then(|_| engine::start(vm).c(d!()))
}

#[inline(always)]
pub(crate) fn zobmie_clean() {
    util::wait_pid()
}

#[cfg(feature = "testmock")]
pub(crate) fn post_clean(_: &Vm) {}

#[inline(always)]
#[cfg(not(feature = "testmock"))]
pub(crate) fn post_clean(vm: &Vm) {
    // 停止 Vm 进程及关联的 Cgroup
    info_omit!(cgroup::kill_vm(vm.id));

    // 清理为 Vm 创建的临时 image
    info_omit!(remove_image(vm));
}

// 命名格式为: ${BASE_IMG}.VmId
#[inline(always)]
#[cfg(feature = "backing_file")]
#[cfg(not(feature = "testmock"))]
fn remove_image(vm: &Vm) -> Result<()> {
    fs::remove_file(format!("{}.{}", vm.image_path.to_string_lossy(), vm.id))
        .c(d!())
}

// 命名格式为: VmId
#[inline(always)]
#[cfg(feature = "zfs_snapshot")]
#[cfg(not(feature = "testmock"))]
fn remove_image(vm: &Vm) -> Result<()> {
    let arg = format!(
        "zfs destroy zroot/pp/{clone_mark}{id}",
        clone_mark = crate::CLONE_MARK,
        id = vm.id()
    );
    cmd_exec("sh", &["-c", &arg]).c(d!())
}

// 执行命令
#[inline(always)]
#[cfg(not(feature = "testmock"))]
fn cmd_exec(cmd: &str, args: &[&str]) -> Result<()> {
    let res = process::Command::new(cmd).args(args).output().c(d!())?;
    if res.status.success() {
        Ok(())
    } else {
        Err(eg!(String::from_utf8_lossy(&res.stderr)))
    }
}
