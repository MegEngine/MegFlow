# 猫猫围栏

## 功能概述
注册的猫猫离开围栏，会收到一条告警信息。未注册的不会报警。

## 模型下载

下载 [google](https://drive.google.com/file/d/1Ff8oxBer135L-wKnkgOewa91lF5EV05P/view?usp=sharing) 解压，软链到 examples/models 目录

```bash
$ cd flow-python/examples
$ ln -s ${DOWNLOAD_DIR}/models models
```

如果有 MegFlow-models repo，可以直接

```bash
$ cd MegFlow-models
$ git-lfs update
$ git lfs pull
```

## 软件安装

启动 redis-server
```bash
$ sudo apt install redis-server
$ redis-server
...
756417:M 18 Aug 2021 17:46:14.641 * Ready to accept connections
```

安装 megengine（1.6.0 引入 lite 接口，简化推理并提供模型加密方案）
```bash
$ python3 -m pip install --upgrade pip
$ python3 -m pip install megengine -f https://megengine.org.cn/whl/mge.html
$ python3
...
>>> import megengine as mge
>>> mge.__version__
'1.6.0...'
```

## 图片注册

启动图片服务
```bash
$ cd flow-python/examples
$ pip3 install -r requires.txt
$ cargo run --example run_with_plugins -- -c cat_finder/image_gpu.toml  -p cat_finder    # 有 GPU 的机器执行这个
$ cargo run --example run_with_plugins -- -c cat_finder/image_cpu.toml  -p cat_finder    # 无 GPU 的 laptop 执行这句
```

服务配置文件在`cat_finder/image_gpu.toml`，详细解释见 [how-to-add-graph](docs/how-to-add-graph.zh.md) 。这里只需要浏览器打开主机所在 8081 端口服务。

```bash
$ google-chrome-stable  http://127.0.0.1:8081/docs 
```

![](images/cat_finder_image_select.jpg)

测试图片在软链接后的 `models/cat_finder_testdata` 目录。打开浏览器 UI 中选择图片、填写名称，提交即可。成功后
* 用 redis-cli `keys *`可查到对应 BASE64 特征
* 前端展示检测框

![](images/cat_finder_image_result.jpg)


## 视频识别

准备一个 rtsp 视频流地址，做测试输入。

* MegFlow 提供了现成的测试地址 `rtsp://10.122.101.175:8554/test1.ts`，可用播放器测试是否可用
* laptop 或树莓派可搜索 Camera 推流教程。见 [如何生成自己的 rtsp 流地址](docs/how-to-generate-rtsp.zh.md)
* 也可以手机拍摄视频，再用 ffmpeg 转成 .ts 格式放到 live555 server。见 [如何生成自己的 rtsp 流地址](docs/how-to-generate-rtsp.zh.md)
* 模型包目录同样提供了测试视频，在`models/cat_finder_testdata`，需要自行部署 live555 服务

启动视频识别服务
```bash
$ cd flow-python/examples
$ cargo run --example run_with_plugins -- -c cat_finder/video_gpu.toml  -p cat_finder  # 有 GPU 的机器
$ cargo run --example run_with_plugins -- -c cat_finder/video_cpu.toml  -p cat_finder  # 无 GPU 的设备用这句
```
服务配置文件在`cat_finder/video_gpu.toml`，详细解释见 [how-to-add-graph](docs/how-to-add-graph.zh.md) 。这里只需要打开 8082 端口服务。

```bash
$ google-chrome-stable  http://127.0.0.1:8082/docs 
```

![](images/cat_finder_video_select.jpg)

`try it out` 其中的 `/start/{url}` 接口，输入 rtsp 地址（例如“rtsp://127.0.0.1:8554/test1.ts”），会返回 stream_id（例如 0）。

* 服务将打印相关日志

```bash
021-08-30 15:17:30.361 | DEBUG    | cat_finder.shaper:exec:82 - shaper recv failed_ids [1]
2021-08-30 15:17:31.760 | DEBUG    | warehouse.detection_yolox.lite:inference:157 - YOLOX infer time: 0.4643s
2021-08-30 15:17:34.094 | INFO     | cat_finder.track:exec:27 - stream tracker finish
2021-08-30 15:17:34.094 | INFO     | cat_finder.shaper:exec:47 - stream shaper finish
2021-08-30 15:17:34.201 | DEBUG    | warehouse.reid_alignedreid.lite:inference:57 - ReID infer time: 0.1072s
2021-08-30 15:17:34.203 | INFO     | cat_finder.redis_proxy:search_key:72 - key: b'feature.laohu' dist: 0.7931023836135864
2021-08-30 15:17:34.203 | INFO     | cat_finder.redis_proxy:search_key:72 - key: b'feature.pingai' dist: 0.5928362607955933
```

* 用 stream_id 做 `/get_msgs/{id}` 的入参，可拉取这路流的解析结果
* redis-cli  `keys *` 可看到报警信息
```bash
$ redis-cli 
127.0.0.1:6379> keys *
1) "feature.laohu"
2) "notification.cat_finder"
3) "feature.pingai"
```
用 `brpop notification.cat_finder` 可消费报警消息。

## 模型列表

本服务有以下模型的痕迹

* [YOLOX](https://github.com/Megvii-BaseDetection/YOLOX)， coco 数据集
* Resnet50，imagenet 数据集。最后一层应用 global pooling
* [AlignedReID](https://arxiv.org/abs/1711.08184)，market1501 数据集。推理阶段移除 local 分支

