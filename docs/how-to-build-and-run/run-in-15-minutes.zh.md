# Run in 15 minutes

## 安装 python3.x （推荐 conda）

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

## 安装 Prebuilt 包

打开 [release 页面](https://github.com/MegEngine/MegFlow/releases)下载对应 python 版本的 .whl 包，安装
```bash
$  python3 -m pip install megflow-0.1.0-py38-none-linux_x86_64.whl  --force-reinstall
```

完成后应该可以 `import megflow`
```bash
$ python3
Python 3.8.3 (default, May 19 2020, 18:47:26) 
[GCC 7.3.0] :: Anaconda, Inc. on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import megflow
```

.whl 提供了 `run_with_plugins`命令，某些环境可能要`export PATH=~/.local/bin/:${PATH}`

```bash
$ apt install build-essential -y
$ run_with_plugins -h
run_with_plugins 1.0
megvii
...
```

## Python“开机自检”

```bash
$ cd ${MegFlow_PATH}/flow-python/examples  # 这行必须
$ run_with_plugins -p logical_test
```

`logical_test` 是 examples 下最基础的计算图测试用例，运行能正常结束表示 MegFlow 编译成功、基本语义无问题。

此处常见问题：`error while loading shared libraries: libpython3.8.xxx`。如果使用 conda 只需要
```bash
$ export LD_LIBRARY_PATH=`conda info --base`/pkgs/python-3.8.11-xxx/lib:${LD_LIBRARY_PATH}
```
`run_with_plugins` 是计算图的实现。使用者不需要关心 Rust/cargo，只需要

  * `import megflow` 成功
  * `run_with_plugins -h` 正常

> 工作原理：[megflow](../../flow-python/megflow/__init__.py) 仅是一层接口，由 run_with_plugins “注入”建图/调度/优化等实现。

## Python Built-in Applications

接下来开始运行好玩的 Python 应用

*  [猫猫围栏运行手册](../built-in-applications/cat_finder.md)
   *  图片注册猫猫
   *  部署视频围栏，注册的猫离开围栏时会发通知
   *  未注册的不会提示
*  [电梯电瓶车告警](../built-in-applications/electric_bicycle.md)
   *  电梯里看到电瓶车立即报警
