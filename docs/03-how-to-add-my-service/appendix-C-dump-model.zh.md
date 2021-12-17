---
id: appendix-A-graph-definition
sidebar_position: 7
---

# 生成 MegEngine 模型

## 生成不带预处理的模型

[Github MegEngine models](https://github.com/MegEngine/models) 有现成的 imagenet 预训模型。这里把模型 dump 成 .mge。

新增 [dump.py](https://github.com/MegEngine/Models/blob/master/official/vision/classification/dump.py)，按 [1, 3, 224, 224] 尺寸 trace 模型，打开推理优化选项，保存为 `model.mge`。

```bash
$ git clone https://github.com/MegEngine/models
$ cd models
$ export PYHTONPATH=${PWD}:${PYTHONPATH}
$ cd official/vision/classification/resnet
$ python3 dump.py
$ ls -lah model.mge
...
```
`dump.py` 已经 PR 到 [MegEngine/models 分类模型目录](https://github.com/MegEngine/Models/tree/master/official/vision/classification)
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
$ cat flow-python/examples/application/simple_classification/lite.py
...
    def inference(self, mat):
        img = self.preprocess(mat, input_size=(224,224), scale_im = False, mean=[103.530, 116.280, 123.675], std=[57.375, 57.120, 58.395])

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

`285` 是分类模型最后一层的 argmax，对应含义需要查[ imagenet 数据集分类表](../../flow-python/examples/application/simple_classification/synset_words.txt) ，这里是 “Egyptian cat”（下标从 0 开始）。

## 生成带预处理的模型

MegEngine 除了不需要转模型，还能消除预处理。我们修改 [dump_resnet.py](https://github.com/MegEngine/MegFlow/blob/master/flow-python/application/examples/misc/dump_resnet.py) 把预处理从 SDK/业务代码提到模型内。这样的好处是：**划清工程和算法的边界**，预处理本来就应该由 scientist 维护，每次只需要 release mge 文件，减少交接内容

```Python
...
    @jit.trace(capture_as_const=True)
    def pred_func(data):
        out = data.astype(np.float32)

        output_h, output_w = 224, 224
        # resize
        print(shape)
        M = mge.tensor(np.array([[1,0,0], [0,1,0], [0,0,1]], dtype=np.float32))        
        M_shape = F.concat([data.shape[0],M.shape])
        M = F.broadcast_to(M, M_shape)
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
