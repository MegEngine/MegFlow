# 串联检测和分类

本文将在 [tutorial01](01-quickstart.zh.md) modelserving 的基础上扩展计算图：先检测、再扣图分类。对外提供视频解析服务。完整的代码在 [simple_det_classify](../../flow-python/examples/simple_det_classify) 。

## 移除分类预处理

见 [生成带预处理的模型](appendix-C-dump-model.zh.md)

## 准备检测模型
这里直接用现成的 YOLOX mge 模型。复用 [cat_finder 的检测](../../flow-python/examples/cat_finder/det.py) 或者从 [YOLOX 官网](https://github.com/Megvii-BaseDetection/YOLOX/tree/main/demo/MegEngine/python) 下载最新版。

##  配置计算图
`flow-python/examples` 增加 `simple_det_classify/video_cpu.toml`

```bash
$ cat flow-python/examples/simple_det_classify/video_cpu.toml

main = "tutorial_02"

# 重资源结点要先声明
[[nodes]]
name = "det"
ty = "Detect"
model = "yolox-s"
conf = 0.25
nms = 0.45
tsize = 640
path = "models/simple_det_classify_models/yolox_s.mge"
interval = 5
visualize = 1
device = "cpu"
device_id = 0

[[nodes]]
name = "classify"
ty = "Classify"
path = "models/simple_det_classify_models/resnet18_preproc_inside.mge"
device = "cpu"
device_id = 0

[[graphs]]
name = "subgraph"
inputs = [{ name = "inp", cap = 16, ports = ["det:inp"] }]
outputs = [{ name = "out", cap = 16, ports = ["classify:out"] }]
# 描述连接关系
connections = [
  { cap = 16, ports = ["det:out", "classify:inp"] },
]

...
# ty 改成 VdieoServer
    [[graphs.nodes]]
    name = "source"
    ty = "VideoServer"
    port = 8085
    
...
```
想对上一期的配置，需要关注 3 点：
* 视频流中的重资源结点，需要声明在  `[[graphs]]` 之外，因为多路视频需要复用这个结点。如果每一路都要启一个 det 结点，资源会爆掉
* `connections` 不再是空白，因为两个结点要描述连接关系
* Server 类型改成 `VideoServer`，告诉 UI 是要处理视频的

## 实现细节
* 可以看到此时 [resnet18 的 lite.py](../../flow-python/examples/simple_det_classify/lite.py) 已经删除了 preprocess 函数
* det.py 可以直接用 `cat_finder` 的

## 运行测试

运行服务
```bash
$ cd flow-python/examples
$ megflow_run -c simple_det_classify/video_cpu.toml  -p simple_det_classify
```
## 资源

[此文档描述 graph toml 定义](appendix-A-graph-definition.zh.md)


[此文档描述 Python node 接口定义](appendix-B-python-plugin.zh.md)