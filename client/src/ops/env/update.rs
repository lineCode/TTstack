//!
//! # Env
//!
//! ```shell
//! pp env update --kick-vm=[OS_PREFIX] --update-life=[SECS] ...
//! ```
//!

use super::super::*;
use crate::{get_servaddr, resp_print};
use myutil::{err::*, *};

///////////////////////////////
#[derive(Default)]
pub struct EnvUpdate<'a> {
    pub life_time: Option<u64>,
    pub os_prefix: Option<Vec<String>>,
    pub env_set: Vec<&'a EnvIdRef>,
    pub is_fucker: bool,
}
///////////////////////////////

impl<'a> EnvUpdate<'a> {
    /// 发送请求并打印结果
    pub fn do_req(&self) -> Result<()> {
        self.env_set.iter().for_each(|env| {
            if let Some(life_time) = self.life_time {
                info_omit!(
                    get_ops_id("update_env_lifetime")
                        .c(d!())
                        .and_then(|ops_id| {
                            get_servaddr().c(d!()).and_then(|addr| {
                                send_req(
                                    ops_id,
                                    gen_req(ReqUpdateEnvLife {
                                        env_id: env.to_string(),
                                        life_time,
                                        is_fucker: self.is_fucker,
                                    }),
                                    addr,
                                )
                                .c(d!())
                            })
                        })
                        .and_then(|resp| resp_print!(resp, String))
                )
            }

            if let Some(os_prefix) = self.os_prefix.as_ref() {
                info_omit!(
                    get_ops_id("update_env_kick_vm")
                        .c(d!())
                        .and_then(|ops_id| {
                            get_servaddr().c(d!()).and_then(|addr| {
                                send_req(
                                    ops_id,
                                    gen_req(ReqUpdateEnvKickVm {
                                        env_id: env.to_string(),
                                        os_prefix: os_prefix.clone(),
                                    }),
                                    addr,
                                )
                                .c(d!())
                            })
                        })
                        .and_then(|resp| resp_print!(resp, String))
                )
            }
        });

        Ok(())
    }
}
