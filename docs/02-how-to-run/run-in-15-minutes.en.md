# Run in 15 minutes

## Install python3.x with conda

Open [miniconda official website](https://docs.conda.io/en/latest/miniconda.html) to download the miniconda installation package, modify the permission and install.

```bash
$ wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh
$ chmod a+x Miniconda3-latest-Linux-x86_64.sh
$ ./Miniconda3-latest-Linux-x86_64.sh
```

Allow conda init the default .bashrc environment  (zsh users also need to modify the conda configuration in .zshrc by self).
```bash
$ conda --version
conda 4.10.3
```

Create a Python3.x (take 3.8 as an example) environment and activate it.
```bash
$ conda create --name py38 python=3.8
$ conda activate py38
```

## Install Prebuilt package
Download the .whl package corresponding to the python version from [MegFlow release](https://github.com/MegEngine/MegFlow/releases), install
```bash
$ python3 -m pip install megflow-0.1.0-py38-none-linux_x86_64.whl --force-reinstall
```

After completion, `import megflow` should works.
```bash
$ python3
Python 3.8.3 (default, May 19 2020, 18:47:26)
[GCC 7.3.0] :: Anaconda, Inc. on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import megflow
```

.whl provides the `megflow_run` command, some environments may require `export PATH=~/.local/bin/:${PATH}`

```bash
$ apt install build-essential -y
$ megflow_run -h
megflow_run 1.0
megvii
...
```

## Power On Self Test

### Download MegFlow source code (we need flow-python/examples)
```bash
$ git clone https://github.com/MegEngine/MegFlow.git
```

### Run Test
```bash
$ cd ${MegFlow_PATH}/flow-python/examples
$ megflow_run -p logical_test
```

`logical_test` is the most basic calculation graph test case. This operation indicates that the MegFlow compilation is successful and there is no problem with the basic semantics.

`megflow_run` is the implementation of computational graphs. Do not need to care about Rust/cargo, just make sure that

  * `import megflow` succeed
  * `megflow_run -h` works well

## Python Built-in Applications

Next, start running Python application

* [cat finder](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/cat_finder)
* [electric bicycle detection](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/electric_bicycle)
* [quickstart](../03-how-to-add-my-service/01-quickstart.zh.md)