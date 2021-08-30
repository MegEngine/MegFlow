# MegFlow
MegFlow 是一个面向视觉应用的流式计算框架, 目标是简单、高性能、帮助机器学习应用快速落地。

## Features
- 基于 [async-std](https://github.com/async-rs/async-std)[features=[tokio1](https://github.com/tokio-rs/tokio)] 的高效异步运行时调度器
- 简洁的基于 [toml](https://toml.io/en/) 的建图描述格式
- 支持静态、动态、共享子图
- 支持rust/python多语言共存
- 支持资源管理(多层级跨任务共享)
- 支持异常处理(异常任务会终止所在静态图)
- 支持demux/reorder/transform等通用函数式组件
- python插件内置有栈协程，不依赖asyncio
- 基础测试工具，支持插件沙盒，用于单测插件
- 基础调试工具，支持建图实时预览/qps profile
  
## HowTo
* [how to build and run in 15 minutes](docs/how-to-build.zh.md)
* [how to add my service](docs/how-to-add-graph.zh.md)
* [how to add plugins](docs/how-to-add-plugins.zh.md)
* [how to optimize and debug](docs/how-to-debug.zh.md)
* [how to contribute](docs/how-to-contribute.zh.md)

## Built-in Applications
* 猫猫围栏
* 电梯电动车报警

## Coming soon
- 进程级别的节点、子图支持
- 插件自动化测试部署
- 性能监控，inspect 等工具
- 更多内置应用和组件

## Contact Us
  * Issue: github.com/MegEngine/MegFlow/issues
  * Email: megengine-support@megvii.com
  * Forum: discuss.megengine.org.cn
  * QQ Group: 1029741705
  * OPENI: openi.org.cn/MegEngine

## [FAQ](docs/FAQ.zh.md)
