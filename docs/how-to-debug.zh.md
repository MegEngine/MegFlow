# 如何 Debug 常见问题

一、`run_with_plugins` 无法启动服务，直接 core dump 报错退出

如果“Python 开机自检”的  `run_with_plugins -p logical_test` 能够正常结束，排查方向应该是 Python import error。调试方法举例
```bash
$  gdb --args ./run_with_plugins  -c electric_bicycle/electric_bicycle_cpu.toml   -p electric_bicycle
...
illegal instruction
...
```
可以看到 crash 发生在哪个 import