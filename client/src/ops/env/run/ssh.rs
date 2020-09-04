//!
//! A slow implementation based on SSH.
//!

use super::VmConnInfo;
use crate::SSH_VM_KEY;
use myutil::{err::*, *};
use std::{
    process::{Child, Command, Stdio},
    sync::mpsc::{channel, Receiver},
    thread,
};

pub const USER: &str = "root";
pub(super) const PORT: u16 = 22;

/// 执行外部的 ssh 命令,
/// 收集远端的输出内容并返回之
#[allow(clippy::mutex_atomic)]
pub(super) fn exec(
    remote_cmd: &str,
    vm_conn_info: Vec<VmConnInfo>,
    _timeout_secs: u64,
) -> Receiver<VmConnInfo> {
    let (s, r) = channel();

    vm_conn_info.into_iter().for_each(|mut vci| {
        let conninfo = format!("{}@{}", USER, &vci.addr);
        let port = vci.ssh_port.to_string();
        let args =
            ["-p".to_owned(), port, conninfo, remote_cmd.to_owned()].to_vec();

        let sender = s.clone();
        thread::spawn(move || {
            exec_run("ssh", args)
                .c(d!())
                .and_then(|child| {
                    child.wait_with_output().c(d!()).map(|output| {
                        vci.stdout = String::from_utf8_lossy(&output.stdout)
                            .into_owned();
                        vci.stderr = String::from_utf8_lossy(&output.stderr)
                            .into_owned();
                    })
                })
                .unwrap_or_else(|e| {
                    vci.stderr = genlog(e);
                });

            info_omit!(sender.send(vci));
        });
    });

    r
}

#[inline(always)]
fn exec_run(cmd: &'static str, args: Vec<String>) -> Result<Child> {
    do_exec(
        &cmd,
        args.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )
}

#[inline(always)]
pub fn do_exec(cmd: &str, args: &[&str]) -> Result<Child> {
    Command::new(cmd)
        .args(&["-o", "ConnectTimeout=3", "-i", SSH_VM_KEY.as_str()])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .c(d!())
}
