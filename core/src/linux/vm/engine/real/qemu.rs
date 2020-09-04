//!
//! # Qemu Virtual Machine
//!

#[cfg(feature = "zfs_snapshot")]
use crate::CLONE_MARK;
use crate::{
    linux::vm::{cmd_exec, util::wait_pid},
    Vm,
};
use lazy_static::lazy_static;
use myutil::{err::*, *};
use std::fs;

const BRIDGE: &str = "ppcore-bridge";
const TAP_SH: &str = "/tmp/.pp_tap.sh";

lazy_static! {
    static ref IOMMU: &'static str = {
        pnk!(
            fs::read("/proc/cpuinfo")
                .c(d!())
                .and_then(|c| String::from_utf8(c).c(d!()))
                .and_then(|cpuinfo| {
                    if cpuinfo.contains(" svm ") {
                        Ok("amd-iommu")
                    } else if cpuinfo.contains(" vmx ") {
                        Ok("intel-iommu")
                    } else {
                        Err(eg!("Unsupported platform!"))
                    }
                })
        )
    };
}

/// 设置基本运行环境
pub(super) fn init() -> Result<()> {
    fs::write("/proc/sys/net/ipv4/ip_forward", "1").c(d!())
        // vhost_net will carry tun/tap/vhost in automaticly
        .and_then(|_| cmd_exec("modprobe", &["vhost_net"]).c(d!()))
        .and_then(|_| {
            cmd_exec("ip", &["addr", "flush", "dev", BRIDGE])
                .c(d!())
                .or_else(|e| {
                    cmd_exec("ip", &["link", "add", BRIDGE, "type", "bridge"])
                        .c(d!(e))
                })
        })
        .and_then(|_| {
            cmd_exec("ip", &["addr", "add", "10.0.0.1/8", "dev", BRIDGE])
                .c(d!())
        })
        .and_then(|_| cmd_exec("ip", &["link", "set", BRIDGE, "up"]).c(d!()))
        .and_then(|_| {
            fs::write(
                TAP_SH,
                format!(
                    "#!/bin/sh\nip link set $1 up;sleep 1;ip link set $1 master {}",
                    BRIDGE
                ),
            )
            .c(d!())
        })
        .and_then(|_| cmd_exec("chmod", &["0544", TAP_SH]).c(d!()))
        .and_then(|_| zfs_snapshot_clean().c(d!()))
}

#[inline(always)]
#[cfg(feature = "backing_file")]
fn zfs_snapshot_clean() -> Result<()> {
    Ok(())
}

#[inline(always)]
#[cfg(feature = "zfs_snapshot")]
fn zfs_snapshot_clean() -> Result<()> {
    let arg = r"
        for i in `zfs list -t all | grep -o 'zroot/pp/clone_[0-9]\+'`; do
            zfs destroy $i || exit 1;
        done;
    ";
    cmd_exec("sh", &["-c", &arg]).c(d!())
}

// **NOTE** 服务启动之前需确保:
// - 主进程运行路径下存在 `tap.sh` 文件
// - 提前创建好 `tap.sh` 文件中使用虚拟网桥
pub(super) fn start(vm: &Vm) -> Result<()> {
    let cpu = vm.cpu_num.to_string();
    let mem = vm.mem_size.to_string();

    let netdev = format!(
        "tap,vhost=on,ifname=ETH-{0},script={1},downscript=no,id=NET_{0}",
        vm.id(),
        TAP_SH
    );

    const WIDTH: usize = 2;
    let netdev_device = format!(
        "virtio-net-pci,mac=52:54:00:11:{:>0width$x}:{:>0width$x},netdev=NET_{}",
        vm.id() / 256,
        vm.id() % 256,
        vm.id(),
        width = WIDTH,
    );

    let (drive, drive_device) = create_img(vm).c(d!())?;

    let args = &[
        "-enable-kvm",
        "-machine",
        "q35,accel=kvm",
        "-device",
        &IOMMU,
        "-cpu",
        "host",
        "-smp",
        cpu.as_str(),
        "-m",
        mem.as_str(),
        "-netdev",
        netdev.as_str(),
        "-device",
        netdev_device.as_str(),
        "-drive",
        drive.as_str(),
        "-device",
        drive_device.as_str(),
        "-boot",
        "order=cd",
        "-vnc",
        &format!(":{}", vm.id()),
        "-daemonize",
    ];

    cmd_exec("qemu-system-x86_64", dbg!(args))
        .map(|_| {
            // Qemu daemonize 模式
            // 会产生一个需要接管的父进程
            wait_pid();
        })
        .c(d!())
}

#[cfg(feature = "zfs_snapshot")]
fn create_img(vm: &Vm) -> Result<(String, String)> {
    let arg = format!(
        "zfs clone zroot/pp/{os}@base zroot/pp/{clone_mark}{id} || exit 1",
        os = vm
            .image_path
            .file_name()
            .ok_or(eg!())?
            .to_str()
            .ok_or(eg!())?,
        clone_mark = CLONE_MARK,
        id = vm.id(),
    );

    cmd_exec("sh", &["-c", &arg]).c(d!()).map(|_|{
        let drive = format!(
            "file=/dev/zvol/zroot/pp/{clone_mark}{id},if=none,format=raw,cache=none,id=DISK_{id}",
            clone_mark = CLONE_MARK,
            id = vm.id(),
        );
        let drive_device = format!("virtio-blk-pci,drive=DISK_{}", vm.id());
        (drive, drive_device)
    })
}

// 基于基础镜像,
// 创建临时运行镜像,
// 命名格式为: ${BASE_IMG}.VmId
#[cfg(feature = "backing_file")]
fn create_img(vm: &Vm) -> Result<(String, String)> {
    let baseimg_path = vm.image_path.to_string_lossy();
    let img_path = format!("{}.{}", baseimg_path, vm.id);

    // **注意**
    //
    // 若指定了 size 选项, 则必须 >= 原始基础镜像,
    // 否则将启动失败, 此处直接不指定递增镜像的大小.
    let option = format!("backing_file={}", baseimg_path);

    let args = &[
        "create",
        "-f",
        "qcow2",
        "-o",
        option.as_str(),
        img_path.as_str(),
    ];

    omit!(fs::remove_file(&img_path));
    cmd_exec("qemu-img", args).c(d!()).map(|_| {
        let drive = format!(
            "file={},if=none,media=disk,id=DISK_{}",
            img_path,
            vm.id()
        );
        let drive_device = format!("virtio-blk-pci,drive=DISK_{}", vm.id());
        (drive, drive_device)
    })
}
