---
id: download-models
sidebar_position: 8
---

# 模型下载

MegFlow 所有模型都压缩成了单个 models.zip 。

| 云盘 | google drive |
| - | - |
| 链接: https://pan.baidu.com/s/1ZLVBR0igJ1hL6PoYQDtByA 提取码: 8ajf | [google](https://drive.google.com/file/d/1EwMJFjNp2kuNglutoleZOVsqccSOW2Z4/view?usp=sharing)  |

取最新的 models_xxx.zip，解压、软链为 examples/models

```bash
$ wget  ${URL}/modes.zip
$ cd flow-python/examples
$ ln -s ${DOWNLOAD_DIR}/models models
```

如果有 MegFlow-models repo，可以直接

```bash
$ cd MegFlow-models
$ git-lfs update
$ git lfs pull
```
