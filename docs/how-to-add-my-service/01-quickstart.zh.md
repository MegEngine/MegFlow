# megflow_quickstart

## 简介

本文介绍如何使用 `megflow_quickstart`  **问答式**创建应用。

目前支持 4 种用法：
* modelserving。单模型图片服务
* 图片 pipeline 服务
* 视频 pipeline
* 自定义模板

## 单模型服务

假设模型使用 megengine 格式且 input tensor 只有一个

```bash
$ megflow_quickstart
...
Welcome to MegFlow quickstart utility.
Please enter values for the following settings (just press Enter to accept a default value, if one is given in brackets).
> Enter the root fullpath for the project. [megflow-app]
megflow-app
> Enter project type, modelserving/image/video/custom? [modelserving]
modelserving
💡   fetching remote template, please wait...
> Enter model input tensor name. [data]
data
> Enter model fullpath. [model.mge]
model.mge
💡   Project created, read ${PROJECT_dir}/README.md to run it.
```

quickstart 会依次问几个问题，并且提供默认值：
* 项目路径
* 服务类型，这里用 modelserving
* input tensor 名称，这里用 data
* 模型所在路径。[阅读此文档生成 megengine 模型](appendix-C-dump-model.zh.md)

正常会提示项目创建成功，阅读 ${PROJECT_dir}/README.md 即可运行。

```bash
$ cd megflow-app
$ ./requires.sh  # 安装 Python 依赖
$ cd ..
$ megflow_run -p megflow-app/config.toml -p megflow-app  # 运行服务
...
# 浏览器打开 127.0.0.1:8080/docs
```

> 对于可恢复的错误（如模板拉取失败），quickstart 会提醒重试，对应 emoji 是 🔧

## 图片/视频服务

```bash
$ megflow_quickstart
...
Welcome to MegFlow quickstart utility.
Please enter values for the following settings (just press Enter to accept a default value, if one is given in brackets).
> Enter the root fullpath for the project. [megflow-app]
megflow-app
> Enter project type, modelserving/image/video/custom? [modelserving]
image
💡   fetching remote template, please wait...
💡   Project created, read ${PROJECT_dir}/README.md to run it.
```

图片/视频创建的项目只有服务框架，可以用 `megflow_run` 直接运行，不含具体业务功能。

## 自定义模板

quickstart 工作原理：
* 拉取 github 上对应分支
* 检查分支里的 placeholder
* 让用户填写 placeholder 对应内容
* 替换 placeholder

此流程同样可用于自定义 repo 和分支，quickstart 提供了 `--git` 参数

```bash
$ megflow_quickstart --git https://github.com/user/repo
...
> Enter project type, modelserving/image/video/custom? [modelserving]
custom
...
```

`custom` 选项会问以下问题：
* 模型路径
* 类型
* 分支名称
* 如有 placeholder，应该替换成什么

placeholder 使用的正则匹配是 
```bash
$ cat flow-quickstart/main.rs
...
    let re = Regex::new(r"##[_\-a-zA-Z0-9]*##").unwrap();
...
```

## MegFlow 服务使用方式

### WebUI 图片
浏览器打开对应端口（例如 http://127.0.0.1:8080/docs ），选择一张图“try it out”即可。

### WebUI 视频
浏览器打开端口服务（例如 http://127.0.0.1:8080/docs ）

* 参照 [如何生成 rtsp](../how-to-build-and-run/generate-rtsp.zh.md)，提供一个 rtsp 流地址
* 或者给 .mp4 文件的绝对路径（文件和 8080 服务在同一台机器上）

### 命令行方式
**图片服务**
```bash
$ curl http://127.0.0.1:8080/analyze/image_name  -X POST --header "Content-Type:image/*"   --data-binary @test.jpeg
```

`image_name` 是用户自定义参数，用在需要 POST 内容的场景。这里随便填即可；`test.jpeg` 是测试图片

**视频服务**
```bash
$ curl  -X POST  'http://127.0.0.1:8085/start/rtsp%3A%2F%2F127.0.0.1%3A8554%2Ftest1.ts'  # start  rtsp://127.0.0.1:8554/test1.ts
start stream whose id is 2% 
$ curl 'http://127.0.0.1:8085/list'   # list all stream
[{"id":1,"url":"rtsp://10.122.101.175:8554/test1.ts"},{"id":0,"url":"rtsp://10.122.101.175:8554/test1.ts"}]%
```
路径中的 `%2F`、`%3A` 是 [URL](https://www.ietf.org/rfc/rfc1738.txt) 的转义字符


### Python Client 方式

[图片 client 代码](../../flow-python/examples/misc/image_client.py)
```Python
import requests
import cv2

def test():
    ip = 'localhost'
    port = '8084'
    url = 'http://{}:{}/analyze/any_content'.format(ip, port)
    img = cv2.imread("./test.jpg")
    _, data = cv2.imencode(".jpg", img)
    data = data.tobytes()

    headers = {'Content-Length': '%d' % len(data), 'Content-Type': 'image/*'}
    res = requests.post(url, data=data, headers=headers)
    print(res.content)

if __name__ == "__main__":
    test()
```


[视频 client 代码](../../flow-python/examples/misc/video_client.py)

```Python

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

### 其他语言
rweb/Swagger 提供了 http RESTful API 描述文件，例如在 http://127.0.0.1:8084/openapi.json 。`swagger_codegen` 可用描述文件生成 java/go 等语言的调用代码。更多教程见 [swagger codegen tutorial ](https://swagger.io/tools/swagger-codegen/)。
