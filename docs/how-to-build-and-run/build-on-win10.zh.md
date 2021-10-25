# Build on win10

## 下载模型包
docker 运行方式，建议把模型包下好，解压备用。[下载地址](../download-models.zh.md)

## 安装 wsl2

[安装文档](https://docs.microsoft.com/zh-cn/windows/wsl/install-win10) 已经非常详细，核心是安装 Linux 内核更新包。完成后第 6 步中的 Linux 分发应该可以正常运行。

## 安装 docker
下载 [windows docker 客户端](https://www.docker.com/products/docker-desktop) 并安装。docker 依赖 wsl2，Docker Desktop 启动正常没有报 fail 即可。

## 安装 git

下载安装 [git 客户端](https://git-scm.com/downloads) 并运行 Git Bash。

```bash
$ pwd
/c/Users/username
$ cd /d  # 切换到合适的盘符
$ git clone https://github.com/MegEngine/MegFlow
...
$ cd MegFlow
$ docker build -t megflow .
... # 等待镜像完成，却决于网络和 CPU
```
> 注意：**不要移动 Dockerfile 文件的位置**。受 [EAR](https://www.federalregister.gov/documents/2019/10/09/2019-22210/addition-of-certain-entities-to-the-entity-list) 约束，MegFlow 无法提供现成的 docker 镜像，需要自己 build 出来，这个过程用了相对路径。

## 运行

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

MegFlow 需要的编译运行环境已完成，接下来开始运行好玩的 Python 应用

*  [猫猫围栏运行手册](../../flow-python/examples/cat_finder/README.md)
   *  图片注册猫猫
   *  部署视频围栏，注册的猫离开围栏时会发通知
   *  未注册的不会提示
*  [电梯电瓶车告警](../../flow-python/examples/electric_bicycle/README.md)
   *  电梯里看到电瓶车立即报警
*  Comming Soon
   *  OCR： 通用字符识别
