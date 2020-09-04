//!
//! # Run cmd in ENV
//!
//! ```shell
//! pp env run ...
//! ```
//!

mod pprexec;
pub mod ssh;

use super::{super::EnvIdRef, show};
use myutil::{err::*, *};
use std::{fs, sync::mpsc::Receiver, time};

///////////////////////////////
#[derive(Default)]
pub struct EnvRun<'a> {
    pub use_ssh: bool,
    pub cmd: &'a str,
    /// 若文件不为空, 则忽略 cmd 项
    pub script: &'a str,
    pub env_set: Vec<&'a EnvIdRef>,
    pub time_out: u64,
}
///////////////////////////////

impl<'a> EnvRun<'a> {
    /// 发送请求并打印结果
    pub fn do_req(&self) -> Result<()> {
        self.get_res().c(d!()).and_then(|(n, r)| {
            for _ in 0..n {
                r.recv_timeout(time::Duration::from_secs(self.time_out))
                    .map(print_to_user)
                    .c(d!())?;
            }
            Ok(())
        })
    }

    /// 发送请求并获取结果
    pub fn get_res(&self) -> Result<(usize, Receiver<VmConnInfo>)> {
        let cmd = if "" != self.script {
            fs::read(self.script)
                .c(d!())
                .and_then(|c| String::from_utf8(c).c(d!()))?
        } else {
            self.cmd.to_owned()
        };

        if "" == cmd {
            return Err(eg!("Empty cmd!"));
        }

        get_conn_info(&self.env_set).c(d!()).map(|vci_set| {
            if self.use_ssh {
                (vci_set.len(), ssh::exec(&cmd, vci_set, self.time_out))
            } else {
                (vci_set.len(), pprexec::exec(&cmd, vci_set, self.time_out))
            }
        })
    }
}

#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct VmConnInfo {
    pub os: String,
    pub addr: String,
    pub ssh_port: u16,
    pub pprexec_port: u16,
    pub stdout: String,
    pub stderr: String,
}

impl VmConnInfo {
    fn new(
        os: String,
        addr: String,
        ssh_port: u16,
        pprexec_port: u16,
    ) -> VmConnInfo {
        VmConnInfo {
            os,
            addr,
            ssh_port,
            pprexec_port,
            stdout: "".to_owned(),
            stderr: "".to_owned(),
        }
    }
}

pub fn print_to_user(r: VmConnInfo) {
    eprintln!(
        "\x1b[35;01m[ {}:{} ] {}\x1b[00m\n\x1b[01m## StdOut ##\x1b[00m\n{}\n\x1b[01m## StdErr ##\x1b[00m\n{}",
        r.addr, r.ssh_port, r.os, r.stdout, r.stderr
    );
}

/// 通过 ENV 查询填充 addr 和 xx_port 字段
pub fn get_conn_info(env_set: &[&EnvIdRef]) -> Result<Vec<VmConnInfo>> {
    show::get_res(env_set).c(d!()).map(|env_set_set| {
        env_set_set
            .into_iter()
            .flat_map(|(_, env_set)| {
                env_set.into_iter().flat_map(|env| {
                    env.vm
                        .into_iter()
                        .filter_map(|(_, vm)| {
                            vm.port_map
                                .get(&ssh::PORT)
                                .copied()
                                .map(|ssh_port| (vm, ssh_port))
                        })
                        .filter_map(|(vm, ssh_port)| {
                            vm.port_map.get(&pprexec::PORT).copied().map(
                                |pprexec_port| {
                                    (vm.os, vm.ip, ssh_port, pprexec_port)
                                },
                            )
                        })
                })
            })
            .map(|(os, ip, ssh_port, pprexec_port)| {
                VmConnInfo::new(os, ip.to_string(), ssh_port, pprexec_port)
            })
            .collect::<Vec<_>>()
    })
}
