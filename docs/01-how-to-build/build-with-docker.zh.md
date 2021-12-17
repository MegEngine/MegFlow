---
id: build-with-docker
sidebar_position: 1
---

# Building with docker

docker build 方式能够“可复现地”生成运行环境、减少依赖缺失的痛苦

## 下载模型包
docker 运行方式，建议把模型包下好，解压备用。[下载地址](../download-models.zh.md)

## 编译 Docker 镜像

```bash
$ cd MegFlow
$ docker build -t megflow -f Dockerfile.github-dev .
```
稍等一段时间（取决于网络和 CPU）镜像构建完成并做了基础自测
> 注意：**不要移动 Dockerfile 文件的位置**。受 [EAR](https://www.federalregister.gov/documents/2019/10/09/2019-22210/addition-of-certain-entities-to-the-entity-list) 约束，MegFlow 无法提供现成的 docker 镜像，需要自己 build 出来，这个过程用了相对路径。
```bash
$ docker images
REPOSITORY            TAG          IMAGE ID       CREATED             SIZE
megflow               latest       c65e37e1df6c   18 hours ago        5.05GB
```
直接用 ${IMAGE ID} 进入开始跑应用，挂载上之前下载好的模型包
```bash
$ docker run  -p 18081:8081 -p 18082:8082 -v ${DOWNLOAD_MODEL_PATH}:/megflow-runspace/flow-python/examples/models -i -t  c65e37e1df6c /bin/bash
```

## Python Built-in Applications

接下来开始运行好玩的 Python 应用

* [猫猫围栏运行手册](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/cat_finder)
   * 图片注册猫猫
   * 部署视频围栏，注册的猫离开围栏时会发通知
   * 未注册的不会提示
* [电梯电瓶车告警](https://github.com/MegEngine/MegFlow/tree/master/flow-python/examples/electric_bicycle)
   * 电梯里看到电瓶车立即报警
* [quickstart](../03-how-to-add-my-service/01-quickstart.zh.md)
   *  问答式创建自己的应用