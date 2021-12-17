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
  * [run with prebuilt .whl](docs/02-how-to-run/run-in-15-minutes.zh.md)
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
* [cat finder](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/cat_finder)
* [electric bicycle detection](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/electric_bicycle)

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

## Acknowledgement

MegFlow examples uses：
* [MegEngine](https://github.com/megengine/megengine)
* [Models](https://github.com/megengine/models)
* [onnx](https://github.com/onnx/onnx)
* [YOLOX](https://github.com/Megvii-BaseDetection/YOLOX)
* [AlignedReID](https://github.com/huanghoujing/AlignedReID-Re-Production-Pytorch)
* [MEMD](https://github.com/megvii-research/MEMD)

MegFlow visualization uses：
* [flv.js](http://bilibili.github.io/flv.js/demo/)

MegFlow Python：
* [OpenCV](https://github.com/opencv/opencv)
* [numpy](https://github.com/numpy/numpy)
* [loguru](https://pypi.org/project/loguru/)
* [scipy](https://github.com/scipy/scipy)
* [redis](https://github.com/redis/redis)

MegFlow Rust refers：
* [anyhow](https://github.com/dtolnay/anyhow)
* [async-std](https://github.com/async-rs/async-std)
* [async-channel](https://github.com/smol-rs/async-channel)
* [clap](https://github.com/clap-rs/clap)
* [concurrent-queue](https://github.com/stjepang/concurrent-queue)
* [ctrlc](https://github.com/Detegr/rust-ctrlc.git)
* [ctor](https://github.com/mmastrac/rust-ctor)
* [dyn-clone](https://github.com/dtolnay/dyn-clone)
* [event-listener](https://github.com/stjepang/event-listener)
* [ffmpeg-next](https://github.com/zmwangx/rust-ffmpeg)
* [hyper](https://github.com/bluss/hyper)
* [headers](https://github.com/bluss/headers)
* [image](https://github.com/image-rs/image)
* [indexmap](https://github.com/bluss/indexmap)
* [lazy-static](https://github.com/rust-lang-nursery/lazy-static.rs)
* [mime](https://github.com/hyperium/mime)
* [numpy](https://github.com/rust-numpy/rust-numpy)
* [oneshot](https://github.com/faern/oneshot)
* [proc-macro2](https://github.com/dtolnay/proc-macro2)
* [pretty-env-logger](https://github.com/seanmonstar/pretty-env-logger)
* [pyo3](https://github.com/pyo3/pyo3)
* [quote](https://github.com/dtolnay/quote)
* [rand](https://github.com/rust-random/rand)
* [rweb](https://github.com/kdy1/rweb)
* [serde](https://github.com/serde-rs/serde)
* [serde_json](https://github.com/serde-rs/json)
* [stackful](https://github.com/nbdd0121/stackful)
* [syn](https://github.com/dtolnay/syn)
* [toml](https://github.com/alexcrichton/toml-rs)
* [urlencoding](https://github.com/kornelski/urlencoding)
* [warp](https://github.com/seanmonstar/warp)
