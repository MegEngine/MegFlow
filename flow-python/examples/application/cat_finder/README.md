---
slug: cat-finder
title: Cat Finder
---

# 猫猫围栏

## 一、功能概述
注册的猫猫离开围栏，会收到一条告警信息。未注册的不会报警。 CPU 配置已提供，没有 GPU 也可以运行。

## 二、[模型和自测数据下载](../../../../docs/download-models.zh.md)

## 三、软件安装

启动 redis-server
```bash
$ sudo apt install redis-server
$ redis-server &
...
756417:M 18 Aug 2021 17:46:14.641 * Ready to accept connections
```

安装 megengine（1.6.0 引入 lite 接口，简化推理并提供模型加密方案）
```bash
$ conda activate py38
$ python3 -m pip install --upgrade pip
$ python3 -m pip install megengine==1.6.0rc1 -f https://megengine.org.cn/whl/mge.html
$ python3
...
>>> import megengine as mge
>>> mge.__version__
'1.6.0'
```

**此处 30xx 卡常见问题**：Ampere 架构的卡安装 megengine，需要看这个 [issue](https://github.com/MegEngine/MegEngine/issues/212)。解决 30xx 强依赖 cuda11 而 mge 又无法分发（EAR 约束）的问题。

## 四、图片注册

启动图片服务
```bash
$ cd flow-python/examples
$ pip3 install -r requires.txt
$ megflow_run -c cat_finder/image_gpu.toml  -p cat_finder    # 有 GPU 的机器执行这个
$ megflow_run -c cat_finder/image_cpu.toml  -p cat_finder    # 无 GPU 的 laptop 执行这句
```

现在 8081 端口部署了“猫体注册”服务，只需要打开浏览器上传图片、猫咪名称即可。`cat_finder/image_gpu.toml` 详细解释见 [how-to-add-graph](../../../../docs/03-how-to-add-my-service/appendix-A-graph-definition.zh.md) 。这里只需要浏览器打开主机所在 8081 端口服务（如 http://127.0.0.1:8081/docs ）。

![](images/cat_finder_image_select.jpg)

测试图片在软链接后的 `models/cat_finder_testdata` 目录。打开浏览器 UI 中选择图片、填写名称，提交即可。成功后
* 用 redis-cli `keys *`可查到对应 BASE64 特征
* 前端展示检测框

![](images/cat_finder_image_result.jpg)

**此处 wsl2 常见问题**：win10 wsl2 用 127.0.0.1 地址打不开服务，**注意 127 是指物理机， wsl2 运行的是虚拟机**，需要 `wsl -- ifconfig` 获取 ip，后面都用虚拟机的 ip。

**此处 docker 常见问题**：服务部署在 docker 里、客户端无法直连，这里提供一些方法：

1） 运行时容器使用端口映射。例如把内部容器的 8081 映射成外部物理机的 18081、把 8082 映射成 18082
```bash
$ docker run -p 18081:8081 -p 18082:8082  -it ubuntu /bin/bash
```
此时用`docker ps`可以看到 PORTS 映射关系
```bash
$ docker ps
CONTAINER ID   IMAGE     COMMAND       CREATED         STATUS         PORTS                                                                                      NAMES
d4b5b563051e   ubuntu    "/bin/bash"   9 seconds ago   Up 8 seconds   0.0.0.0:18081->8081/tcp, :::18081->8081/tcp, 0.0.0.0:18082->8082/tcp, :::18082->8082/tcp   nostalgic_swartz

```
浏览器打开宿主机的 ip:18081 端口即可使用

2）在容器内用 `cURL` 发 HTTP POST 请求，不再用 web UI
```bash
$ curl http://127.0.0.1:8081/analyze/my_cat_name  -X POST --header "Content-Type:image/*"   --data-binary @test.jpeg  --output out.jpg
```
`my_cat_name` 是注册的猫咪名称；`test.jpeg` 是测试图片；`output.jpg` 是返回的可视化图片。

3）Python 提供了调用参照 [image_client.py](https://github.com/MegEngine/MegFlow/blob/master/flow-python/examples/application/misc/image_client.py) 

## 五、准备视频识别

启动解析服务
```bash
$ cd flow-python/examples
$ megflow_run -c cat_finder/video_gpu.toml  -p cat_finder  # 有 GPU 的机器
$ megflow_run -c cat_finder/video_cpu.toml  -p cat_finder  # 无 GPU 的设备用这句
```
浏览器打开 8082 端口服务（如 http://127.0.0.1:8082/docs ，注意区分物理机和虚拟机的对应 ip）

![](images/cat_finder_video_select.jpg)

可以看到 MegFlow 提供了 4 个 API：启/停一路解析、消费当前解析结果、列出所有信息。

开启一路视频流解析需要流的 url，这里有两种方法：

1）准备一个 rtsp 视频流地址，做测试输入（流地址部署不方便，也可以直接用离线文件的绝对路径代替）。模型包目录提供了测试视频，在 `models/cat_finder_testdata`，需要用户自行部署 live555 服务。最直接的办法：
```bash
$ wget https://github.com/aler9/rtsp-simple-server/releases/download/v0.17.2/rtsp-simple-server_v0.17.2_linux_amd64.tar.gz
$ 
$ tar xvf rtsp-simple-server_v0.17.2_linux_amd64.tar.gz && ./rtsp-simple-server 
$ ffmpeg -re -stream_loop -1 -i ${models}/cat_finder_testdata/test1.ts -c copy -f rtsp rtsp://127.0.0.1:8554/test1.ts
```

* 想用 laptop/树莓派摄像头可搜索 Camera 推流教程
* 也可以手机拍摄视频，再用 ffmpeg 转成 .ts 格式推到 live555 server

相关教程已整合在 [如何生成自己的 rtsp 流地址](../../../../docs/02-how-to-run/generate-rtsp.zh.md) 。

2）如果 rtsp 流地址部署不方便，也可以直接用离线文件的绝对路径代替，也就是在 WebUI 中输入类似`/mnt/data/stream/file.ts` 的路径。需要自行保证服务器可访问这个文件、并且格式是可以被 ffmpeg 解析的（例如 .ts/.mp4/.h264/.h265）。

## 六、视频识别 FAQ

* 如果用 wsl2 部署，注意区分物理机和虚拟机的 ip
* 如果服务部署在 docker 里，同样可以把 8082 端口映射到宿主机端口

## 七、运行

### 1）WebUI 方式

`try it out` 其中的 `/start/{url}` 接口，输入 rtsp 地址（如“rtsp://127.0.0.1:8554/test1.ts”或者“/home/name/file.mp4”），会返回 stream_id（例如 0）。

* 服务将打印相关日志

```bash
2021-08-30 15:17:30.361 | DEBUG    | cat_finder.shaper:exec:82 - shaper recv failed_ids [1]
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
用 `rpop notification.cat_finder` 可消费报警消息。

### 2）命令行方式
对应的 `cURL` 命令参考
```bash
$ curl  -X POST  'http://127.0.0.1:8082/start/rtsp%3A%2F%2F127.0.0.1%3A8554%2Ftest1.ts'  # start  rtsp://127.0.0.1:8554/test1.ts
start stream whose id is 2% 
$ curl 'http://127.0.0.1:8082/list'   # list all stream
[{"id":1,"url":"rtsp://10.122.101.175:8554/test1.ts"},{"id":0,"url":"rtsp://10.122.101.175:8554/test1.ts"}]%
```
路径中的 `%2F`、`%3A` 是 [URL](https://www.ietf.org/rfc/rfc1738.txt) 的转义字符

### 3）Python 代码方式
参照 [video_client.py](https://github.com/MegEngine/MegFlow/blob/master/flow-python/examples/application/misc/video_client.py) 实现

## 八、 Web 实时展示解析结果

参考 [tutorial04](../../../../docs/03-how-to-add-my-service/04-web-visualization.zh.md) 操作说明
