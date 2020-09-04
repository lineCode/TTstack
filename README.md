# pp

pp, Private Platform.

快速生成各种虚拟机环境, 为产品兼容性验证和自动化测试提供高效的基础环境, 主要用作 SAAS 产品的测试私有云.

## 主要用途

1. 广泛的平台兼容性验证
    - 可在如下两个方向上做任意的交叉组合
        1. Linux、BSD、Windows、MacOS 等各种 OS 类别与版本
        2. AMD64、X86、AArch64、ARM、MIPS、RISC-V、SPARC 等各种硬件平台
2. 与 DevOps 系统配合, 实现自动化的 CI/CD 功能
3. 用作原生编译平台
    - 直接申请全量的原生 OS 环境, 避免交叉编译的复杂度和潜在问题
4. 用作短期或长期的调试环境
    - 可将 PP 视为云平台, 申请虚拟机用于开发和测试
5. 其它...

## 技术特性

- 整洁高效的资源管理
    - (Linux) 每个 VM 存在于独立的 Cgroup 中, 资源清理准确无误
    - (Linux) 使用 Qemu 的`backing_file`机制管理 VM 镜像, 小巧快速
    - (Linux) [可选] 使用 zfs 的 `snapshot + clone` 机制使 VM 获得原生 IO 性能
    - (Linux) 使用 nftables 的 `SET/MAP` 等高级数据结构管理网络端口
    - (Linux) 服务进程运行在单独的`PID NS`中, 服务退出会自动销毁所有资源
    - (Linux) 服务进程注册自身为`SUBREAPER`, 确保 zombie 进程会被及时清理
    - (FreeBSD) 使用比 Qemu 更高效的 Bhyve 管理 VM 实例
    - (FreeBSD) 使用 zfs 的 `snapshot + clone` 机制使 VM 获得原生 IO 性能
    - 通过`Rust Drop`机制管理 VM 的生命周期
    - ...
- 分布式可扩展架构
    - 后端支持多机分布式架构, 对用户完全透明
- 轻量级的通信模型
    - C/S 两端基于 UDP 进行通信
    - 自研的远程命令执行工具(pprexec), 效率远超 SSH 协议
- 镜像源与服务解耦
    - 可随时增加受支持的系统镜像, 服务端不需要停机
- 使用`Rust`语言开发
    - 安全稳定
    - 高效运行
    - 文档齐备
    - 原生跨平台
    - ...

## 详细文档

- [用户指南](./documents/user_guide.md)
- [系统管理指南](./documents/system_admin.md)
- [架构设计与技术选型](./documents/arch_design.md)
- [项目结构与代码规模](./documents/code_about.md)

> #### 接口文档
>
> ```shell
> # 在 Rust 开发环境下执行
> make doc
> ```
