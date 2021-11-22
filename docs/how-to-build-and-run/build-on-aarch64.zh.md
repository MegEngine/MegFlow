# aarch64 源码编译

## 一、环境安装

所需依赖包查看 Dockerfile，若已安装可跳过

```bash
$ cat Dockerfile.github-dev
...
# install requirements
RUN apt-get update
RUN apt-get install -y wget yasm clang git build-essential
RUN apt install -y libssl-dev
RUN apt update && apt-get install -y pkg-config --fix-missing
RUN apt-get install -y curl
...
```

**如果是华为鲲鹏 ARM 服务器， gcc 需要 >= 7.5 版本，默认的 `aarch64-redhat-linux-gcc 4.8.5`  缺 `__ARM_NEON` 没法用** 
```bash
$ yum install -y centos-release-scl
$ yum install -y devtoolset-8-gcc devtoolset-8-gcc-c++
$ source /opt/rh/devtoolset-8/enable 
$ gcc --version
gcc (GCC) 8.3.1 20190311 (Red Hat 8.3.1-3)
...
```

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
$ wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-aarch64.sh
$ chmod a+x Miniconda3-latest-Linux-aarch64.sh
$ ./Miniconda3-latest-Linux-aarch64.sh
```

安装时接受 conda 修改默认 .bashrc 环境变量（zsh 用户还需自行修改 .zshrc 中的 conda initialize 配置）。成功后 `conda` 可正常运行
```bash
$ conda --version
conda 4.10.3
```

创建一个 Python3.x（这里以 3.8 为例） 的环境，激活。
```bash
$ conda create --name py38 python=3.8
$ conda activate py38
```


## 三、编译

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


## 四、Python“开机自检”
```bash
$ cd examples
$ cargo build --example megflow_run --release # 编译出 megflow bin
$ ln -s ../../target/example/megflow_run
$ ./megflow_run -p logical_test
```
`logical_test` 是 examples 下最基础的计算图测试用例，运行能正常结束表示 MegFlow 编译成功、基本语义无问题。

`megflow_run` 是计算图的实现。编译完成之后不再需要 `cargo` 和 `Rust`，使用者只需要

  * `import megflow`成功
  * `megflow_run -h` 正常


## 五、Python Built-in Applications

接下来开始运行好玩的 Python 应用

*  [猫猫围栏运行手册](../built-in-applications/cat_finder.md)
   *  图片注册猫猫
   *  部署视频围栏，注册的猫离开围栏时会发通知
   *  未注册的不会提示
*  [电梯电瓶车告警](../built-in-applications/electric_bicycle.md)
   *  电梯里看到电瓶车立即报警

## 六、其他选项
```bash
$ cargo run --example graph -- ${args} # 测试 MegFlow 的延迟/吞吐/调度开销, 更多使用说明通过--help 查看
$ cargo run --example megflow_run -- ${args} # 基于插件 + 参数文件形式运行 MegFlow, 更多说明通过--help 查看
$ export RUST_LOG=LOG_LEVEL // 设置日志级别, 例如 INFO, TRACE..
$ cargo build [--release] // 编译
$ cargo check // 快速编译，不执行 link
$ cargo test [target] // 执行单元测试
```
