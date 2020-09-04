//!
//! # Network Service Handler
//!
//! Operations to deal with the requests from client.
//!

pub(crate) mod server;

use super::{send_err, send_ok, SERV};
use crate::{def::*, CFG};
use myutil::{err::*, *};
use ppcore::{Env, VmCfg};
use serde::Deserialize;
use std::{
    mem,
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

type Ops = fn(SocketAddr, Vec<u8>) -> Result<()>;
include!("../../../server_def/src/included_ops_map.rs");

/// 基于 peeraddr 生成 CliId
#[inline(always)]
fn gen_cli_id(peeraddr: &SocketAddr) -> String {
    peeraddr.ip().to_string()
}

// 注册 Cli, 一般无需调用,
// 创建 Env 时若发现 Cli 不存在会自动创建之
fn register_client_id(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    let mut req = serde_json::from_slice::<Req<&str>>(&request).c(d!())?;
    SERV.add_client(req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)))
        .c(d!())
        .and_then(|_| send_ok!(req.uuid, "Success!", peeraddr).c(d!()))
        .or_else(|e| send_err!(req.uuid, e, peeraddr).c(d!()))
}

/// 获取服务端的资源分配信息
fn get_server_info(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    let req = serde_json::from_slice::<Req<&str>>(&request).c(d!())?;

    let rsc = SERV.get_resource();
    let res = RespGetServerInfo {
        vm_total: rsc.vm_active,
        cpu_total: rsc.cpu_total as u32,
        cpu_used: rsc.cpu_used,
        mem_total: rsc.mem_total as u32,
        mem_used: rsc.mem_used,
        disk_total: rsc.disk_total as u32,
        disk_used: rsc.disk_used,
        supported_list: server::OS_INFO.read().keys().cloned().collect(),
    };

    send_ok!(req.uuid, map! {CFG.serv_at.clone() => res}, peeraddr)
}

/// 获取服务端已存在的 Env 概略信息
fn get_env_list(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    let mut req = serde_json::from_slice::<Req<&str>>(&request).c(d!())?;
    let res = SERV.get_env_meta(
        &req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)),
    );
    send_ok!(req.uuid, map! {CFG.serv_at.clone() => res}, peeraddr).c(d!())
}

/// 获取服务端已存在的 Env 详细信息
fn get_env_info(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    #[derive(Deserialize)]
    struct MyReq {
        uuid: u64,
        cli_id: Option<CliId>,
        msg: ReqGetEnvInfo,
    }

    let mut req = serde_json::from_slice::<MyReq>(&request).c(d!())?;
    let mut envinfo = SERV.get_env_detail(
        &req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)),
        req.msg.env_set,
    );

    // VM 的私有地址替换为服务器的地址
    envinfo.iter_mut().for_each(|env| {
        env.vm.values_mut().for_each(|vm| {
            vm.ip = Ipv4::new(CFG.serv_ip.clone());
        });
    });

    send_ok!(req.uuid, map! {CFG.serv_at.clone() => envinfo}, peeraddr).c(d!())
}

#[derive(Default)]
struct ReqAddEnvWrap(ReqAddEnv);

impl Deref for ReqAddEnvWrap {
    type Target = ReqAddEnv;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReqAddEnvWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ReqAddEnvWrap {
    fn check_dup(&self) -> Result<u8> {
        const DUP_MAX: u8 = 64;
        let dup_each = self.0.dup_each.unwrap_or(0);
        if DUP_MAX < dup_each {
            Err(eg!(format!(
                "the number of `dup` too large: {}(max {})",
                dup_each, DUP_MAX
            )))
        } else {
            Ok(dup_each)
        }
    }

    // 自动添加 SSH/pprexec 端口影射
    fn set_ssh_port(&mut self) {
        self.0.port_set.push(22); // SSH
        self.0.port_set.push(22000); // pprexec
        self.0.port_set.sort();
        self.0.port_set.dedup();
    }

    fn set_os_lowercase(&mut self) {
        self.0
            .os_prefix
            .iter_mut()
            .for_each(|os| *os = os.to_lowercase());
    }

    /// 根据请求参数生成 Env
    pub fn create_env(mut self) -> Result<Env> {
        self.set_ssh_port();
        self.set_os_lowercase();
        let me = &self;
        let dup_each = self.check_dup().c(d!())?;
        let mut env = Env::new(&SERV, &self.0.env_id).c(d!())?;
        let vmset = server::OS_INFO
            .read()
            .iter()
            .filter(|(os, _)| {
                self.0.os_prefix.iter().any(|pre| os.starts_with(pre))
            })
            .map(|(_, img_path)| {
                (0..(1 + dup_each)).map(move |_| VmCfg {
                    image_path: img_path.to_owned(),
                    port_list: me.0.port_set.clone(),
                    kind: me.0.vm_kind,
                    cpu_num: me.0.cpu_num,
                    mem_size: me.0.mem_size,
                    disk_size: me.0.disk_size,
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        env.update_life(self.0.life_time.unwrap_or(3600), false)
            .c(d!())?;
        env.add_vm_set(vmset).c(d!())?;

        Ok(env)
    }
}

/// 创建新的 Env
fn add_env(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    #[derive(Deserialize)]
    struct MyReq {
        uuid: u64,
        cli_id: Option<CliId>,
        msg: ReqAddEnv,
    }

    let mut req = serde_json::from_slice::<MyReq>(&request).c(d!())?;
    let id = req.uuid;
    ReqAddEnvWrap(mem::take(&mut req.msg))
        .create_env()
        .c(d!())
        .and_then(|env| {
            SERV.register_env(
                req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)),
                env,
            )
            .c(d!())
        })
        .and_then(|_| send_ok!(id, "Success!", peeraddr).c(d!()))
        .or_else(|e| send_err!(id, e, peeraddr).c(d!()))
}

/// 从已有 ENV 中踢出指定的 VM 实例
fn update_env_kick_vm(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    #[derive(Deserialize)]
    struct MyReq {
        uuid: u64,
        cli_id: Option<CliId>,
        msg: ReqUpdateEnvKickVm,
    }

    let mut req = serde_json::from_slice::<MyReq>(&request).c(d!())?;
    let cli_id = req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr));
    SERV.get_env_detail(&cli_id, vct![req.msg.env_id.clone()])
        .into_iter()
        .for_each(|ei| {
            info_omit!(
                SERV.update_env_del_vm(
                    &cli_id,
                    &ei.id,
                    ei.vm
                        .iter()
                        .filter(|(_, vm)| {
                            req.msg.os_prefix.iter().any(|prefix| {
                                vm.os
                                    .to_lowercase()
                                    .starts_with(&prefix.to_lowercase())
                            })
                        })
                        .map(|(&id, _)| id)
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
            );
        });

    send_ok!(req.uuid, "Success!", peeraddr).c(d!())
}

/// 更新已有 Env 的生命周期
fn update_env_lifetime(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    #[derive(Deserialize)]
    struct MyReq {
        uuid: u64,
        cli_id: Option<CliId>,
        msg: ReqUpdateEnvLife,
    }

    let mut req = serde_json::from_slice::<MyReq>(&request).c(d!())?;
    SERV.update_env_life(
        &req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)),
        &req.msg.env_id,
        req.msg.life_time,
        req.msg.is_fucker,
    )
    .c(d!())
    .and_then(|_| send_ok!(req.uuid, "Success!", peeraddr).c(d!()))
    .or_else(|e| send_err!(req.uuid, e, peeraddr).c(d!()))
}

/// 删除 Env
fn del_env(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    #[derive(Deserialize)]
    struct MyReq {
        uuid: u64,
        cli_id: Option<CliId>,
        msg: ReqDelEnv,
    }

    let mut req = serde_json::from_slice::<MyReq>(&request).c(d!())?;
    SERV.del_env(
        &req.cli_id.take().unwrap_or_else(|| gen_cli_id(&peeraddr)),
        &req.msg.env_id,
    );

    send_ok!(req.uuid, "Success!", peeraddr).c(d!())
}

/// 获取服务端已存在的 Env 概略信息(全局)
fn get_env_list_all(peeraddr: SocketAddr, request: Vec<u8>) -> Result<()> {
    let req = serde_json::from_slice::<Req<&str>>(&request).c(d!())?;
    let res = SERV.get_env_meta_all();
    send_ok!(req.uuid, map! {CFG.serv_at.clone() => res}, peeraddr).c(d!())
}
