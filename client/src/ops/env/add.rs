//!
//! # Env
//!
//! ```shell
//! pp env add ...
//! ```
//!

use super::super::*;
use crate::{get_servaddr, resp_print};
use myutil::{err::*, *};

///////////////////////////////
#[derive(Default)]
pub struct EnvAdd<'a> {
    pub env_id: &'a str,
    pub os_prefix: Vec<&'a str>,
    /// 暂时不支持自定义, 此选项将被忽略
    #[allow(dead_code)]
    pub vm_type: Option<&'a str>,
    pub vm_port: Vec<u16>,
    /// 0 代表使用服务端预设的默认值
    pub life_time: u64,
    /// 0 代表使用服务端预设的默认值
    pub cpu_num: u32,
    /// 0 代表使用服务端预设的默认值
    pub mem_size: u32,
    /// 0 代表使用服务端预设的默认值
    pub disk_size: u32,
    /// 0 代表使用服务端预设的默认值
    pub dup_each: u8,
}
///////////////////////////////

impl<'a> EnvAdd<'a> {
    /// 发送请求并打印结果
    pub fn do_req(self) -> Result<()> {
        get_ops_id("add_env")
            .c(d!())
            .and_then(|ops_id| {
                get_servaddr().c(d!()).and_then(|addr| {
                    send_req(ops_id, gen_req(ReqAddEnv::from(self)), addr)
                        .c(d!())
                })
            })
            .and_then(|resp| resp_print!(resp, String))
    }
}

impl<'a> From<EnvAdd<'a>> for ReqAddEnv {
    fn from(v: EnvAdd<'a>) -> Self {
        ReqAddEnv {
            env_id: v.env_id.to_owned(),
            vm_kind: None,
            os_prefix: v.os_prefix.into_iter().map(|s| s.to_owned()).collect(),
            life_time: alt!(0 == v.life_time, None, Some(v.life_time)),
            cpu_num: alt!(0 == v.cpu_num, None, Some(v.cpu_num)),
            mem_size: alt!(0 == v.mem_size, None, Some(v.mem_size)),
            disk_size: alt!(0 == v.disk_size, None, Some(v.disk_size)),
            port_set: v.vm_port,
            dup_each: alt!(0 == v.dup_each, None, Some(v.dup_each)),
        }
    }
}
