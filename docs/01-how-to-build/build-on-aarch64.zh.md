# aarch64 源码编译

aarch64 源码编译方式和 [源码编译](build-from-source.zh.md) 相同，此处只说明差异。

## 环境差异

如果是华为鲲鹏 ARM 服务器 CentOS 系统， gcc 需要 >= 7.5 版本，系统默认的 `aarch64-redhat-linux-gcc 4.8.5`  缺 `__ARM_NEON` 会导致大量异常。
```bash
$ yum install -y centos-release-scl
$ yum install -y devtoolset-8-gcc devtoolset-8-gcc-c++
$ source /opt/rh/devtoolset-8/enable 
$ gcc --version
gcc (GCC) 8.3.1 20190311 (Red Hat 8.3.1-3)
...
```
## 软件差异

conda 建议使用 `archiconda`，目前（2021.12.06） miniconda aarch64 官网版本在 KhadasVIM3/JetsonNao 上均会崩溃。`archiconda` 安装：
```bash
$ wget https://github.com/Archiconda/build-tools/releases/download/0.2.3/Archiconda3-0.2.3-Linux-aarch64.sh
$ chmod +x Archiconda3-0.2.3-Linux-aarch64.sh && ./Archiconda3-0.2.3-Linux-aarch64.sh
```
