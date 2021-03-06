# Building from Source

## 一、安装依赖

### 安装 Rust
```bash
$ sudo apt install curl
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

成功后，`cargo` 应该可以正常执行
```bash
$ cargo --version
cargo 1.56.0 (4ed5d137b 2021-10-04)
```

如果不成功，提示`Command 'cargo' not found`，可以按照提示加载一下环境变量(重新连接或打开终端也可以)：
```
source $HOME/.cargo/env
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

创建一个 Python3.x（这里以 3.8 为例） 的环境，激活。
```bash
$ conda create --name py38 python=3.8
$ conda activate py38
```

### 安装基础依赖库

megflow的构建需要一些基础依赖库, 下面以ubuntu系统为例:
```bash
$ sudo apt update
$ sudo apt install -y wget yasm clang git build-essential
$ sudo apt install -y libssl-dev
$ sudo apt install -y pkg-config --fix-missing
```


## 二、编译

编译并安装Python包到当前用户下

```bash
$ git clone --recursive https://github.com/MegEngine/MegFlow --depth=1
$ cd MegFlow/flow-python
$ python3 setup.py install --user
```
编译成功后，在 Python `import megflow` 正常。

## 三、Python“开机自检”
```bash
$ cd examples
$ megflow_run -p logical_test
```
`logical_test` 是 examples 下最基础的计算图测试用例，运行能正常结束表示 MegFlow 编译成功、基本语义无问题。

`megflow_run` 是计算图的实现。编译完成之后不再需要 `cargo` 和 `Rust`，使用者只需要

  * `import megflow` 成功
  * `megflow_run -h` 正常

## 四、[编译选项](appendix-A-build-options.md)
