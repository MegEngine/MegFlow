# 把分类模型变成服务

尽管 MegFlow 解决的是多个（20+）模型组织成 pipeline 的问题，但凡事总要一步步来。本文介绍如何 step by step 集成 1 个分类模型，最终成为图片/视频 http 服务。

## 准备分类模型

[MegEngine models]() 有现成的 imagenet 预训模型。这里把模型 dump 成 .mge。

新增 [dump.py](../../flow-python/examples/simple_classification/dump.py)，按 [1, 3, 224, 224] 尺寸 trace 模型，打开推理优化选项，保存为 `model.mge`。

```bash
$ git clone https://github.com/MegEngine/models
$ cd models
$ export PYHTONPATH=${PWD}:${PYTHONPATH}
$ cd official/vision/classification/resnet
$ python3 dump.py
$ ls -lah model.mge
...
```
`dump.py` 正在 PR 到 MegEngine/models
```bash
$ cat dump.py
...
    data = mge.Tensor(np.random.random((1, 3, 224, 224)))  # 准备一个样例输入

    @jit.trace(capture_as_const=True)
    def pred_func(data):
        outputs = model(data)  # trace 每个 opr 的 shape
        return outputs

    pred_func(data)
    pred_func.dump( # 保存模型
        graph_name,
        arg_names=["data"],
        optimize_for_inference=True, # 打开推理优化选项
        enable_fuse_conv_bias_nonlinearity=True, # 打开 fuse conv+bias+ReLU pass 推理更快
    )
...
```

## 模型单测
开发模型的推理封装，对外提供[功能内聚](https://baike.baidu.com/item/%E9%AB%98%E5%86%85%E8%81%9A%E4%BD%8E%E8%80%A6%E5%90%88/5227009)的接口。调用方传入一张或多张图片、直接获取结果，尽量避免关心内部实现（如用何种 backbone、预处理是什么、后处理是什么）。

```bash
$ cat flow-python/examples/simple_classification/lite.py
...
    def inference(self, mat):
        # 设置输入
        inp_data =self.net.get_io_tensor("data")
        inp_data.set_data_by_share(img)
        
        # 推理
        self.net.forward()
        self.net.wait()

        # 取输出
        output_keys = self.net.get_all_output_name()
        output = self.net.get_io_tensor(output_keys[0]).to_numpy()
        return np.argmax(output[0])
...
$ python3 lite.py --model model.mge  --path test.jpg  # 测试
2021-09-14 11:45:02.406 | INFO     | __main__:<module>:81 - 285
```

`285` 是分类模型最后一层的 argmax，对应含义需要查[ imagenet 数据集分类表](../../flow-python/examples/simple_classification/synset_words.txt) ，这里是 “Egyptian cat”（下标从 0 开始）。

##  配置计算图
`flow-python/examples`增加`simple_classification/image_cpu.toml`

```bash
$ cat flow-python/examples/simple_classification/image_cpu.toml
main = "tutorial_01_image"

[[graphs]]
name = "subgraph"
inputs = [{ name = "inp", cap = 16, ports = ["classify:inp"] }]  # 一、输入输出结点
outputs = [{ name = "out", cap = 16, ports = ["classify:out"] }]
connections = [
]

    [[graphs.nodes]] # 二、结点参数
    name = "classify"
    ty = "Classify"
    path = "models/simple_classification_models/resnet18.mge"
    device = "cpu"
    device_id = 0
...
    [[graphs.nodes]] # 三、服务类型配置
    name = "source"
    ty = "ImageServer"
    port = 8084 # 端口号 8084
    response = "json"
...
```
开发时直接从别处复制一个过来即可，图片单模型服务只需要关心 3 处
* 计算图输入、输出结点的名字。这里是`classify`
* `classify` 结点的参数。最重要的是 `ty="Classify"`指明了类名，MegFlow 将在当前目录搜索`Classify`类。path/device/device_id 分别是模型路径/用 CPU 推理/用哪个核，属于用户自定义配置
* 服务类型。这里想运行图片服务 `ty = "ImageServer"`，如果想运行视频解析服务改 `ty = "VideoServer"`；图片服务默认返回图片，想返回 string 需要配置 `response = "json"`

[完整的计算图 config 定义](appendix-A-graph-definition.md)

## 实现配置中的 node

创建文件`classify.py`，把之前实现的模型推理调起来即可
```bash
$ cat flow-python/examples/simple_classification/classify.py
...
@register(inputs=['inp'], outputs=['out'])
class Classify:
    def __init__(self, name, args):
        logger.info("loading Resnet18 Classification...")
        self.name = name
        
        # load model and warmup
        self._model = PredictorLite(path=args['path'], device=args['device'], device_id=args['device_id'])
        warmup_data = np.zeros((224, 224, 3), dtype=np.uint8)
        self._model.inference(warmup_data)
        logger.info("Resnet18  loaded.")

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        data = envelope.msg['data']
        result = self._model.inference(data)
        self.out.send(envelope.repack(json.dumps(str(result))))
```
实现只有 2 点：
* `__init__` 里加载模型，做个 warmup 防止首次推理太慢
* 解码成 BGR 的 data 在 `envelope.msg['data']`，推理，send 返回 json string

[更多 node 说明](appendix-B-python-plugin.zh.md)

## 运行测试

运行服务
```bash
$ cd flow-python/examples
$ cargo run --example run_with_plugins -- -c simple_classification/image_cpu.toml  -p  simple_classification # 源码/docker 编译方式用这条命令
```

浏览器打开 8084 端口服务（例如 http://10.122.101.175:8084/docs ），选择一张图“try it out”即可。

## 其他

一、http 客户端开发

rweb/Swagger 提供了 http RESTful API 描述文件，例如在 http://10.122.101.175:8084/openapi.json 。`swagger_codegen` 可用描述文件生成各种语言的调用代码。更多教程见 [swagger codegen tutorial ](https://swagger.io/tools/swagger-codegen/)。