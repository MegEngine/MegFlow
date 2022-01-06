# 打包成 Python .whl

## 作用
打成 whl 包，使用方直接安装即可，不再需要编译。

## 执行

现在使用 Dockerfile 生成各 python 版本 .whl

```bash
$ cd ${MegFlow_dir}
$ # 构造开发环境，安装依赖。已执行过 docker 编译可以跳过此步骤
$ docker build -t megflow -f Dockerfile.github-dev .
$ # 创建结果目录
$ mkdir dist
$ # docker 打包  whl
$ # https://stackoverflow.com/questions/33377022/how-to-copy-files-from-dockerfile-to-host
$ DOCKER_BUILDKIT=1 docker build -f Dockerfile.github-release --output dist .
```

**注意** COPY to host 需要：
* Docker 19.03 以上版本
* 需要 DOCKER_BUILDKIT 环境变量
* 需要 --output 参数