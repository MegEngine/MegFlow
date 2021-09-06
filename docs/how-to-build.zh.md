# Building from Source

## Prerequisites

### 软硬件环境

| 测试通过的环境 | 备注 |
| - | - |
| win10 WSL ubuntu18.04 | - |
| x86 Ubuntu16.04 服务器有 GPU | - |
| x86 Ubuntu18.04 无 GPU | 运行时选 CPU config |

支持主流 x86 Linux 版本，ARM 还在开发。

### 安装 Rust
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

成功后，`cargo` 应该可以正常执行
```bash
$ cargo --version
cargo 1.53.0 (4369396ce 2021-04-27)
```

> `cargo` 是 Rust 的包管理器兼编译辅助工具。类似 Java maven/go pkg/C++ CMake 的角色，更易使用。

### 安装 python3.x （推荐 conda）

打开 [miniconda 官网](https://docs.conda.io/en/latest/miniconda.html) 下载 miniconda 安装包，修改权限并安装。

```bash
$ wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh
$ chmod a+x Miniconda3-latest-Linux-x86_64.sh
$ ./Miniconda3-latest-Linux-x86_64.sh
```

安装时接受 conda 修改默认 .bashrc 环境变量（zsh 用户还需自行修改 .zshrc 中的 conda initialize 配置）。成功后 `conda` 可正常运行
```bash
$ conda --version
conda 4.10.3
```

创建一个 Python3.8（已测试 3.6.13/3.7.11/3.8.11 可用。**3.9 暂不可用**，这里以 3.8 为例）的环境，激活。
```bash
$ conda create --name py38 python=3.8
$ conda activate py38
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
$ git clone --recursive https://github.com/MegEngine/MegFlow --depth=1
$ cd MegFlow
$ cargo build
waiting ...
$ cd flow-python
$ python3 setup.py install --user
```

P.S. 默认 ffmpeg 依赖自动从 github 上拉取源码构建，这会使得首次构建的时间较长。若希望缩短首次构建时间，或者希望依赖一个指定版本的 ffmpeg，可以启用环境变量`CARGO_FEATURE_PREBUILD`并参考[rust-ffmpeg](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building)自行构建

## Python “开机自检”用例
```bash
$ cd examples
$ cargo run --example run_with_plugins -- -p logical_test
```
`logical_test` 是 examples 下最基础的计算图测试用例，运行能正常结束表示 MegFlow 编译成功、基本语义无问题。

此处常见问题：`error while loading shared libraries: libpython3.8.xxx`。如果使用 conda 只需要
```bash
$ export LD_LIBRARY_PATH=/home/`whoami`/miniconda3/pkgs/python-3.8.11-h12debd9_0_cpython/lib:${LD_LIBRARY_PATH}
```

## Python Built-in Applications

接下来开始运行好玩的 Python 应用

*  [猫猫围栏运行手册](../flow-python/examples/cat_finder/README.md)
   *  图片注册猫猫
   *  部署视频围栏，注册的猫离开围栏时会发通知
   *  未注册的不会提示
*  [电梯电瓶车告警](../flow-python/examples/electric_bicycle/README.md)
   *  电梯里看到电瓶车立即报警
*  Comming Soon
   *  OCR： 通用字符识别


## 其他选项
```bash
$ cargo run --example graph -- ${args} # 测试 MegFlow 的延迟/吞吐/调度开销, 更多使用说明通过--help 查看
$ cargo run --example run_with_plugins -- ${args} # 基于插件 + 参数文件形式运行 MegFlow, 更多说明通过--help 查看
$ export RUST_LOG=LOG_LEVEL // 设置日志级别, 例如 INFO, TRACE..
$ cargo build [--release] // 编译
$ cargo check // 快速编译，不执行 link
$ cargo test [target] // 执行单元测试
```
