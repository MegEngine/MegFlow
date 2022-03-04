# 视频实时超分

## 一、[下载模型和测试数据](../../../../docs/download-models.zh.md)

## 二、运行

模型软链后，使用`megflow_run`运行
```bash
$ cd ${path/to/MegFlow}/flow-python/examples/application # 这行必须
$ megflow_run  -c video_super_resolution/config.toml  -p video_super_resolution
```

浏览器打开 [8087 端口](http://10.122.101.175:8087/docs#/default/post_start__url_)，try it out。

1）上传测试数据中的 `a.mp4`，获取 stream_id

2）用 stream_id 查询结果存到了哪个文件，例如 kj2WAS.flv

## 三、如何使用自己的模型

超分模型使用 [basicVSR_mge](https://github.com/Feynman1999/basicVSR_mge) 做训练，炼丹结束后请 cherry-pick  [这个 PR](https://github.com/Feynman1999/basicVSR_mge/pull/6)，然后把模型 jit.trace 成纯推理格式。

```bash
$ cd ${path/to/basicVSR_mge}
$ python3 tools/dump.py configs/restorers/BasicVSR/basicVSR_test_valid.py
```

## 四、其他
构造测试输入需**注意视频质量**，例如
```bash
$ ffmpeg -i images/%08d.png -q:v 2  -vcodec mpeg4 -r 20  a.mp4
```
