# 批量推理和 Pipeline 级测试

本文将在 [tutorial02](02-det-attr.zh.md) 的基础上扩展功能：动态 batching 测试 QPS 提升。

## 分类模型支持动态 batch

resnet 的 dump 需要支持多 batch 输入，样例 [dump_resnet.py](../../flow-python/examples/misc/dump_resnet.py)

## 分类用 batch_recv 接口

新的 classify.py 改成这样：
```bash
$ cat flow-python/examples/simple_det_classify/classify.py
...
    def exec(self):
        # batching
        (envelopes, _) = self.inp.batch_recv(self.batch_size, self.timeout)

        if len(envelopes) == 0:
            return
...
```
这里 `batch_recv` 的参数列表

| 类型 | 名称 | 含义 |
| -----  | ------ | -----  |
| 输入 | batch_size | 最多攒多少 batch |  
| 输入 | timeout | 多少毫秒内返回 |
| 输出 | list<Any> | 一组消息，0 <= len(list) <= batch_size |
| 输出 | bool | 标识该端口是否已经关闭，当其值为 True 时，语义等同于 `recv` 接口返回的None |


然后在 Python 层合并 data，调 `inference_batch`
```Python
data = np.concatenate(crops)
types = self._model.inference_batch(data)
```

## Pipeline 级测试
MegFlow 支持直接输入图片集/视频列表做测试，不需要 http 服务。使用方自行实现 Validation 结点，集成进 CI 做正确性/性能测试。

### 图片集测试
以 [simple_classification image_test](../../flow-python/examples/simple_classification/image_test.toml) 为例
```bash
...
    [[graphs.nodes]]
    name = "source"
    ty = "ImageInput"
    urls = ["/mnt/data/user/image/","/home/test_data_dir/"]
...
```
pipeline 建图等不变，新增了一种 source 叫做 `ImageInput`，调用方填 `urls` 做图片目录列表。

运行方法不变
```bash
$ megflow_run -c simple_classification/image_test.toml  -p simple_classification
```

### 视频列表测试
以 [simple_det_classify video_test](../../flow-python/examples/simple_det_classify/video_test.toml) 为例：
```bash
...
    [[graphs.nodes]]
    name = "source"
    ty = "VideoInput"
    repeat = 1
    urls = ["rtsp://127.0.0.1:8554/test.ts", "/mnt/data/file.mp4"]
...
```

建图同样不变，新增了 `VideoInput` 结点，参数列表

| 参数 | 含义 |
| -----  | ------ |
| urls | 视频 url 列表，流地址、本地文件皆可 |  
| repeat | 每个 url 并行创建多少路。注意**如果是网络流，需要调用方考量带宽压力** |

使用方法不变
```bash
$ megflow_run  -c simple_det_classify/video_test.toml   -p simple_det_classify
```
