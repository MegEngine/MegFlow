<div align="center">
  <img width="60%"  src="logo.png">
</div>

<small> [简体中文](README.md) | English </small>

## MegFlow | [Documentation](https://megflow.readthedocs.io/zh_CN/latest/#)
[![GitHub license](https://img.shields.io/badge/license-apache--2--Clause-brightgreen.svg)](./LICENSE)
[![ubuntu](https://img.shields.io/github/workflow/status/MegEngine/MegFlow/ubuntu-x86-cpu?label=ubuntu)](https://github.com/MegEngine/MegFlow/actions/workflows/ubuntu-x86-cpu.yml?query=workflow%3A)
[![macos](https://img.shields.io/github/workflow/status/MegEngine/MegFlow/ubuntu-x86-cpu?label=macos)](https://github.com/MegEngine/MegFlow/actions/workflows/macos-x86-cpu.yml?query=workflow%3A)

Build video analysis service in 15 minutes. 

* Directly use Python to build pipeline
* No need C++ SDK anymore, improve the development experience
* Provide one-stop service for construction, testing, debugging, deployment, and visualization

## HowTo
* how to run
  * [run with prebuilt .whl](docs/02-how-to-run/run-in-15-minutes.en.md)
  * [generate RTSP](docs/02-how-to-run/generate-rtsp.zh.md)
* how to build
  * [build with docker](docs/01-how-to-build/build-with-docker.zh.md)
  * [build from source](docs/01-how-to-build/build-from-source.zh.md)
  * [build with win10 wsl](docs/01-how-to-build/build-on-win10.zh.md)
  * [build on armv8](docs/01-how-to-build/build-on-aarch64.zh.md)
* how to use
  * [tutorial01: quickstart](docs/03-how-to-add-my-service/01-quickstart.zh.md)
  * [tutorial02: detect and classify on video stream](docs/03-how-to-add-my-service/02-det-attr.zh.md)

  * [tutorial03: batching and pipeline test](docs/03-how-to-add-my-service/03-batching-and-pipeline-test.zh.md)
  * [tutorial04: visualization](docs/03-how-to-add-my-service/04-web-visualization.zh.md)
* [how to debug](docs/how-to-debug.zh.md)
* [how to contribute](docs/how-to-contribute.zh.md)
* [FAQ](docs/FAQ.zh.md)

## Current Support Matrix

| Platform | win10 docker/wsl2 | ubuntu | centOS | macos |
| ----------- | ------------------------- | ---------- | ---------- | --------- |
| x86 | ✔️ | ✔️ | ✔️ | ✔️ |  
| ARMv8 | - | ✔️ | ✔️ | - |

| Python verion | support |
| ----------- | -------- |
| 3.6         | ✔️        |
| 3.7         | ✔️        |
| 3.8         | ✔️        |
| 3.9         | ✔️        |

## Built-in Applications
* [cat finder](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/cat_finder)
* [electric bicycle detection](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/electric_bicycle)
* [video super resolution](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/application/video_super_resolution)

## Features
- Efficient runtime schedule based on [async-std](https://github.com/async-rs/async-std)[features=[tokio1](https://github.com/tokio-rs/tokio)]
- Use [toml](https://toml.io/en/) to construct pipeline
- Support static/dynamic/share subgraph
- Support Rust and Python
- Support resource management
- Terminate static subgraph in exception processing
- Support demux/reorder/transform
- Use Python stackfull coroutine
- Support plugin sandbox
- Real-time preview constructing pipeline

## Coming Soon
- Process-level node
- Plug-in automated test
- Performance monitoring
- More built-in applications

## Contact Us
  * Issue: github.com/MegEngine/MegFlow/issues
  * Email: megengine-support@megvii.com
  * QQ Group: 1029741705

## License
- [Apache 2.0](LICENSE)
