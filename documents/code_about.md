# pp

pp 主要代码使用 Rust 编写.

Rust 是一门风格**紧凑**、运行**高效**的现代开发语言, 以**最少的代码量**实现**最高的性能**, 打破了几十年来的旷世难题: 静态语言与动态语言两者的优势不能兼得.

## 项目结构

```
$ (git)-[master]-% tree -F -I 'target' -L 1
.
|-- Cargo.toml           # 项目配置文件
|-- client/              # 客户端 pp 命令的代码实现
|-- core/                # 服务端的核心逻辑实现
|-- core_def/            # 从 core 模块中提取出的通用定义, 供 server 模块使用
|-- documents/           # 项目详细文档
|-- LICENSE
|-- Makefile
|-- proxy/               # 分布式架构后端, 负责统筹调度多个 Server 的资源
|-- README.md            # 项目主文档
|-- rexec/               # 一个轻量级的"远程命令执行和文件转输"方案
|-- rustfmt.toml
|-- rust_toolchain
|-- server/              # 后端 Server 的代码实现, 可独立运行, 也可挂靠在 Proxy 之后
|-- server_def/          # 从 server 模块中提取出的通用定义, 供 proxy 模块使用
`-- tools/               # 外围脚本工具
```

## 代码规模

```
$ (git)-[master]-% find . -type f | grep -Ev 'target|\.(git|lock)' | xargs wc -l | grep -Ev '^ +[0-9]{1,2} '

   158 ./core/src/linux/mod.rs
   204 ./core/src/linux/vm/engine/real/qemu.rs
   143 ./core/src/linux/vm/cgroup/real.rs
   164 ./core/src/linux/nat/mod.rs
   629 ./core/src/def.rs
   212 ./core/src/freebsd/vm/mod.rs
   278 ./server/src/hdr/mod.rs
   163 ./server/tests/standalone/mod.rs
   241 ./server/tests/knead/mod.rs
   138 ./server_def/src/lib.rs
   368 ./client/src/cmd_line.rs
   156 ./client/src/cfg_file.rs
   116 ./client/src/ops/mod.rs
   129 ./client/src/ops/env/run/mod.rs
   259 ./rexec/src/server.rs
   161 ./rexec/src/common.rs
   119 ./rexec/src/bin/cli.rs
   148 ./rexec/src/client.rs
   176 ./rexec/tests/integration.rs
   152 ./core_def/src/lib.rs
   101 ./proxy/src/def.rs
   171 ./proxy/src/hdr/add_env.rs
   342 ./proxy/src/hdr/mod.rs
   171 ./proxy/src/lib.rs
   133 ./proxy/tests/env/mod.rs
   179 ./proxy/tests/standalone/mod.rs
   282 ./proxy/tests/knead/mod.rs
   332 ./documents/user_guide.md
  9105 total
```
