# Building from Source

## Prerequisites

### 软硬件环境

*nix 系统（Linux/Mac），x86 芯片。

* 普通 laptop 可用 megenginelite CPU 模式运行，每个 application 的 README 会提供 CPU config
* MegFlow 原理上支持 ARM，coming soon

### 安装 Rust
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

成功后，`cargo` 应该可以正常执行
```bash
$ cargo --version
cargo 1.53.0 (4369396ce 2021-04-27)
```

> `cargo` 是 Rust 的包管理器兼编译辅助工具。类似 Java maven/ go pkg/ C++ CMake 的角色，更易使用。

### 安装 python3.8 （推荐 conda）

打开 [miniconda 官网](https://docs.conda.io/en/latest/miniconda.html) 下载 miniconda 安装包，修改权限并安装。

```bash
$ wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh
$ chmod a+x Miniconda3-latest-Linux-x86_64.sh
$ ./Miniconda3-latest-Linux-x86_64.sh
```

安装时接受 conda 修改默认 .bashrc 环境变量（zsh 用户还需自行修改 .zshrc 中的 conda initialize 配置）。成功后 `conda` 可正常运行
```
$ conda --version
conda 4.10.3
```

创建一个 Python3.8 的环境，激活。
```bash
$ conda create --name py38 python=3.8
$ conda activate py38
```

## Clone
```bash
$ git clone --recursive https://github.com/MegEngine/MegFlow --depth=1
$ cd MegFlow
$ git submodule update
```

## Build

MegFlow 需要编译 ffmpeg。考虑到 ffmpeg 依赖较多、本身又是常用工具，最简单的办法就是直接装 ffmpeg 把编译依赖装上

```bash
$ sudo apt install yasm  # ffmpeg 编译依赖
$ sudo apt install ffmpeg
$ ffmpeg 
ffmpeg version 3.4.8...
$ sudo apt install clang
$ clang --version
clang version 6.0.0-1ubuntu2...
```

编译底层 Rust 组件，安装 Python module 

```bash
$ cd MegFlow
$ cargo build
$ cd flow-python
$ python3 setup.py install --user
```

P.S. 默认 ffmpeg 依赖自动从 github 上拉取源码构建，这会使得首次构建的时间较长。若希望缩短首次构建时间，或者希望依赖一个指定版本的 ffmpeg，可以启用环境变量`CARGO_FEATURE_PREBUILD`并参考[rust-ffmpeg](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building)自行构建

## Python Examples 基础用例
```bash
$ cd examples
$ cargo run --example run_with_plugins -- -p logical_test
```
`logical_test` 就是 examples 下的测试用例名称，默认使用和目录同名的`logical_test.toml`做配置文件。

此处常见问题：`error while loading shared libraries: libpython3.8.xxx`。如果使用 conda 只需要
```bash
$ export LD_LIBRARY_PATH=/home/`whoami`/miniconda3/pkgs/python-3.8.11-h12debd9_0_cpython/lib:${LD_LIBRARY_PATH}
```

## Python Built-in Applications

*  [猫猫围栏运行手册](flow-python/examples/cat_finder/README.zh.md)
   *  图片注册猫猫
   *  部署视频围栏，注册的猫猫离开围栏时会发通知
   *  未注册的不会提示
*  [电梯电瓶车告警](flow-python/examples/electric_bicycle/README.zh.md)
   *  电梯里看到电瓶车立即报警
*  Comming Soon
   *  OCR： 通用字符识别

## Rust Examples 格式
```bash
$ cargo run --example graph -- ${args} # 测试 MegFlow 的延迟/吞吐/调度开销, 更多使用说明通过--help 查看
$ cargo run --example run_with_plugins -- ${args} # 基于插件 + 参数文件形式运行 MegFlow, 更多说明通过--help 查看
```

## Development
```bash
$ export RUST_LOG=LOG_LEVEL // 设置日志级别, 例如 INFO, TRACE..
$ cargo build [--release] // 编译
$ cargo check // 快速编译，不执行 link
$ cargo test [target] // 执行单元测试
```