mod docker;
mod qemu;

use crate::{Vm, VmKind};
use myutil::{err::*, *};

// TODO: support more vm-engine
#[inline(always)]
pub(in crate::linux) fn start(vm: &Vm) -> Result<()> {
    match vm.kind {
        VmKind::Qemu => qemu::start(vm).c(d!()),
        VmKind::Docker => docker::start(vm).c(d!()),
        _ => Err(eg!("Unsupported VmKind!")),
    }
}

#[inline(always)]
pub(in crate::linux) fn init() -> Result<()> {
    qemu::init().c(d!())
}
