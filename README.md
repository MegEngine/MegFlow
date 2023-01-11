<div align="center">
  <img width="60%"  src="logo.png">
</div>

<small> 简体中文 | [English](README.en.md) </small>

## MegFlow [Documentation](https://megflow.readthedocs.io/zh_CN/latest/)
[![GitHub license](https://img.shields.io/badge/license-apache--2--Clause-brightgreen.svg)](./LICENSE)
![ubuntu](https://img.shields.io/github/actions/workflow/status/megengine/megflow/ubuntu-x86-cpu.yml?branch=master)
![macos](https://img.shields.io/github/actions/workflow/status/megengine/megflow/macos-x86-cpu.yml?branch=master)

MegFlow 提供快速视觉应用落地流程，最快 15 分钟搭建起视频分析服务。其特性体现在：

* 直接用 Python 搭建计算图（如先检测、再跟踪、最后质量判断加识别），不必关心 C++、图优化相关问题
* 省去 SDK 集成、提升开发体验，通过流程改进应对人力不足、时间紧、功能多的情况
* 提供 pipeline 搭建、测试、调试、部署、结果可视化一条龙服务

## HowTo
* 如何运行
  * [使用预编译 .whl](docs/02-how-to-run/run-in-15-minutes.zh.md)
  * [生成 RTSP 地址](docs/02-how-to-run/generate-rtsp.zh.md)
* 如何编译
  * [docker 编译](docs/01-how-to-build/build-with-docker.zh.md)
  * [ubuntu 源码编译](docs/01-how-to-build/build-from-source.zh.md)
  * [win10 wsl 编译](docs/01-how-to-build/build-on-win10.zh.md)
  * [armv8 编译](docs/01-how-to-build/build-on-aarch64.zh.md)

* 构建自己的 pipeline
  * [tutorial01: quickstart 问答式创建应用](docs/03-how-to-add-my-service/01-quickstart.zh.md)
  * [tutorial02: detect and classify on video stream](docs/03-how-to-add-my-service/02-det-attr.zh.md)
  * [tutorial03: batching and pipeline test](docs/03-how-to-add-my-service/03-batching-and-pipeline-test.zh.md)
  * [tutorial04: visualization](docs/03-how-to-add-my-service/04-web-visualization.zh.md)
* [how to debug](docs/how-to-debug.zh.md)
* [how to contribute](docs/how-to-contribute.zh.md)
* [FAQ](docs/FAQ.zh.md)

## Current Support Matrix

| 系统环境 | win10 docker/wsl2 | ubuntu | centOS | macos |
| ----------- | ------------------------- | ---------- | ---------- | --------- |
| x86 | ✔️ | ✔️ | ✔️ | ✔️ |  
| ARMv8 | - | ✔️ | ✔️ | - |

| Python 版本 | 支持情况 |
| ----------- | -------- |
| 3.6         | ✔️        |
| 3.7         | ✔️        |
| 3.8         | ✔️        |
| 3.9         | ✔️        |

## Built-in Applications
* [猫猫围栏](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/cat_finder)
* [电梯电动车报警](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/electric_bicycle)
* [视频实时超分](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/video_super_resolution)

## Features
- 基于 [async-std](https://github.com/async-rs/async-std)[features=[tokio1](https://github.com/tokio-rs/tokio)] 的高效异步运行时调度器
- 简洁的基于 [toml](https://toml.io/en/) 的建图描述格式
- 支持静态、动态、共享子图
- 支持 Rust/Python 多语言共存（会 Python 即可）
- 支持资源管理（多层级跨任务共享）
- 支持异常处理（异常任务会终止所在静态图）
- 支持 demux/reorder/transform 等通用函数式组件
- Python 插件内置有栈协程，不依赖 asyncio
- 基础测试工具，支持插件沙盒，用于单测插件
- 基础调试工具，支持建图实时预览/qps profile

## Coming Soon
- 进程级别的节点、子图支持
- 插件自动化测试部署
- 性能监控，inspect 等工具
- 更多内置应用和组件

## Contact Us
  * Issue: github.com/MegEngine/MegFlow/issues
  * Email: megengine-support@megvii.com
  * QQ Group: 1029741705

## License
- [Apache 2.0](LICENSE)
