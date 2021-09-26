# 串联检测和分类

本文将在 [tutorial01](01-single-classification-model.zh.md) 的基础上扩展计算图：先检测、再扣图分类。对外提供视频解析服务。完整的代码在 [simple_det_classify](../../flow-python/examples/simple_det_classify) 。

## 移除分类预处理

之前提到过：MegEngine 除了不需要转模型，还能消除预处理。我们修改 `dump.py` 把预处理从 SDK/业务代码提到模型内。这样的好处是：**划清工程和算法的边界**，预处理本来就应该由 scientist 维护，每次只需要 release mge 文件，减少交接内容

```bash
$ cat ${MegFlow}/flow-python/examples/simple_det_classify/dump.py
...
    data = mge.Tensor(np.ones(shape, dtype=np.uint8))

    @jit.trace(capture_as_const=True)
    def pred_func(data):
        out = data.astype(np.float32)
        # resnet18 预处理
        output_h, output_w = 224, 224
        # resize
        M = mge.tensor(np.array([[1,0,0], [0,1,0], [0,0,1]], dtype=np.float32).reshape((1,3,3)))
        out = F.vision.warp_perspective(out, M, (output_h, output_w), format='NHWC')
        # mean
        _mean = mge.Tensor(np.array([103.530, 116.280, 123.675], dtype=np.float32))
        out = F.sub(out, _mean)
        # div 
        _div = mge.Tensor(np.array([57.375, 57.120, 58.395], dtype=np.float32))
        out = F.div(out, _div)
        # dimshuffile 
        out = F.transpose(out, (0,3,1,2))

        outputs = model(out)
        return outputs
...
```
具体实现是在 trace inference 里增加预处理动作，fuse opr 优化加速的事情交给 MegEngine 即可。更多 cv 操作参照 [MegEngine API 文档](https://megengine.org.cn/doc/stable/zh/reference/api/megengine.functional.vision.warp_perspective.html?highlight=warp_perspective)。

因为推理输入变成了 BGR，所以 dump 模型的时候参数也应该跟着变
```bash
$ python3 dump.py -a resnet18 -s 1 224 224 3 
```

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
$ run_with_plugins -c simple_det_classify/video_cpu.toml  -p simple_det_classify
```

### WebUI 方式
浏览器打开 8085 端口服务（例如 http://127.0.0.1:8085/docs ）

* 参照 [如何生成 rtsp](../how-to-build-and-run/generate-rtsp.zh.md)，提供一个 rtsp 流地址
* 或者干脆给 .mp4 文件的绝对路径（文件和 8085 服务在同一台机器上）

### 命令行方式
```bash
$ curl  -X POST  'http://127.0.0.1:8085/start/rtsp%3A%2F%2F127.0.0.1%3A8554%2Ftest1.ts'  # start  rtsp://127.0.0.1:8554/test1.ts
start stream whose id is 2% 
$ curl 'http://127.0.0.1:8085/list'   # list all stream
[{"id":1,"url":"rtsp://10.122.101.175:8554/test1.ts"},{"id":0,"url":"rtsp://10.122.101.175:8554/test1.ts"}]%
```
路径中的 `%2F`、`%3A` 是 [URL](https://www.ietf.org/rfc/rfc1738.txt) 的转义字符

### Python Client

```Python
$ cat ${MegFlow_DIR}/flow-python/examples/simple_det_classify/client.py

import requests
import urllib


def test():
    ip = 'localhost'
    port = '8085'
    video_path = 'rtsp://127.0.0.1:8554/vehicle.ts'
    video_path = urllib.parse.quote(video_path, safe='')
    url = 'http://{}:{}/start/{}'.format(ip, port, video_path)

    res = requests.post(url)
    ret = res.content
    print(ret)


if __name__ == "__main__":
    test()
```
