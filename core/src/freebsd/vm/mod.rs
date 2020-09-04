//!
//! # Virtual Machine Mgmt
//!
//! - Bhyve
//! - Jail
//! - ...
//!

#[cfg(not(feature = "testmock"))]
use super::CLONE_MARK;
use crate::Vm;
use myutil::err::*;
#[cfg(not(feature = "testmock"))]
use myutil::*;
#[cfg(not(feature = "testmock"))]
use nix::unistd::{daemon, execv, fork, ForkResult};
#[cfg(not(feature = "testmock"))]
use std::{ffi::CString, process};

#[cfg(feature = "testmock")]
pub(super) fn init() -> Result<()> {
    Ok(())
}

// 清理旧数据,
// 是否应由启动脚本去做?
#[cfg(not(feature = "testmock"))]
pub(super) fn init() -> Result<()> {
    let arg = r"
        kldload ipfw ipfw_nat if_bridge if_tap 2>/dev/null;
        sysctl net.link.tap.up_on_open=1 || exit 1;

        ifconfig bridge0 destroy 2>/dev/null;
        ifconfig bridge0 create up || exit 1;

        ifconfig bridge0 inet 10.0.0.1/8 -alias 2>/dev/null;
        ifconfig bridge0 inet 10.0.0.1/8 alias || exit 1;
    ";

    cmd_exec("sh", &["-c", arg])
        .c(d!())
        .and_then(|_| env_clean().c(d!()))
}

#[cfg(feature = "testmock")]
pub(super) fn env_clean() -> Result<()> {
    Ok(())
}

// 清理 VM 环境
#[cfg(not(feature = "testmock"))]
pub(super) fn env_clean() -> Result<()> {
    let arg = r"
        for i in `ls /dev/vmm | grep -o '^[0-9]\+'`; do
            bhyvectl --destroy --vm=$i || exit 1;
        done;
        for i in `zfs list -t all | grep -o 'zroot/pp/clone_[0-9]\+'`; do
            zfs destroy $i || exit 1;
        done;
    ";

    cmd_exec("sh", &["-c", arg]).c(d!())
}

#[inline(always)]
#[cfg(feature = "testmock")]
pub(crate) fn start(_: &Vm) -> Result<()> {
    Ok(())
}

#[inline(always)]
#[cfg(not(feature = "testmock"))]
pub(crate) fn start(vm: &Vm) -> Result<()> {
    // zfs destroy 动作有延迟,
    // 在 init 中统一清理, 此处不再处理
    let pre_arg = format!(
        "
        ifconfig tap{id} destroy 2>/dev/null;
        ifconfig tap{id} create || exit 1;
        ifconfig bridge0 addm tap{id} up || exit 1;

        bhyvectl --destroy --vm={id} 2>/dev/null;
        zfs clone zroot/pp/{os}@base zroot/pp/{clone_mark}{id} ||  exit 1;
        ",
        id = vm.id(),
        os = vm
            .image_path
            .file_name()
            .ok_or(eg!())?
            .to_str()
            .ok_or(eg!())?,
        clone_mark = CLONE_MARK,
    );

    cmd_exec("sh", &["-c", &pre_arg])
        .c(d!())
        .and_then(|_| bhyve_exec(vm).c(d!()))
}

#[cfg(not(feature = "testmock"))]
fn bhyve_exec(vm: &Vm) -> Result<()> {
    let id = vm.id().to_string();
    let cpu = vm.cpu_num.to_string();
    let mem = format!("{}M", vm.mem_size);
    let disk =
        format!("2,virtio-blk,/dev/zvol/zroot/pp/{}{}", CLONE_MARK, &id);

    const WIDTH: usize = 2;
    let nic = format!(
        "3,virtio-net,tap{id},mac=00:be:fa:76:{aa:>0width$x}:{bb:>0width$x}",
        id = &id,
        aa = vm.id() / 256,
        bb = vm.id() % 256,
        width = WIDTH,
    );

    let args = &[
        "-A",
        "-H",
        "-P",
        "-c",
        &cpu,
        "-m",
        &mem,
        "-s",
        "0,hostbridge",
        "-s",
        "1,lpc",
        "-s",
        &disk,
        "-s",
        &nic,
        "-l",
        "bootrom,/usr/local/share/uefi-firmware/BHYVE_UEFI.fd",
        &id,
    ];

    start_vm("/usr/sbin/bhyve", dbg!(args)).c(d!())
}

// 必须后台执行
#[inline(always)]
#[cfg(not(feature = "testmock"))]
fn start_vm(cmd: &str, args: &[&str]) -> Result<()> {
    let cmd = gen_cstring(cmd);
    let args = args.iter().map(|arg| gen_cstring(arg)).collect::<Vec<_>>();

    match fork() {
        Ok(ForkResult::Child) => daemon(false, false)
            .c(d!())
            .and_then(|_| {
                execv(
                    &cmd,
                    &args
                        .as_slice()
                        .iter()
                        .map(|arg| arg.as_ref())
                        .collect::<Vec<_>>(),
                )
                .c(d!())
            })
            .map(|_| ()),
        Ok(_) => Ok(()),
        Err(e) => Err(e).c(d!()),
    }
}

#[inline(always)]
#[cfg(not(feature = "testmock"))]
fn gen_cstring(s: &str) -> CString {
    unsafe { CString::from_vec_unchecked(s.as_bytes().to_vec()) }
}

// Nothing to do on freebsd.
pub(crate) fn zobmie_clean() {}

#[cfg(feature = "testmock")]
pub(crate) fn post_clean(_: &Vm) {}

// 清理过程中出错继续执行
//     - 请理 `/dev/vmm` 下的 VM 名称占用
//     - 清理 VM 的临时 `clone` 镜像
//         - 路径为: zroot/pp/${VM_ID}
//     - 清理 tap${VM_ID} 网络设备
#[inline(always)]
#[cfg(not(feature = "testmock"))]
pub(crate) fn post_clean(vm: &Vm) {
    let arg = format!(
        "
        bhyvectl --destroy --vm={id};
        zfs destroy zroot/pp/{clone_mark}{id};
        ifconfig bridge0 deletem tap{id};
        ifconfig tap{id} destroy;
        ",
        id = vm.id(),
        clone_mark = CLONE_MARK,
    );
    info_omit!(cmd_exec("sh", &["-c", &arg]));
}

// 执行命令
#[inline(always)]
#[cfg(not(feature = "testmock"))]
pub(super) fn cmd_exec(cmd: &str, args: &[&str]) -> Result<()> {
    let res = process::Command::new(cmd).args(args).output().c(d!())?;

    if res.status.success() {
        Ok(())
    } else {
        Err(eg!(String::from_utf8_lossy(&res.stderr)))
    }
}
