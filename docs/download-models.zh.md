# 模型和测试数据下载

| 应用名称 | 云盘 | google drive |
| - | - | - |
| 猫猫围栏、电动车检测 | 链接: https://pan.baidu.com/s/1ZLVBR0igJ1hL6PoYQDtByA 提取码: 8ajf | [google](https://drive.google.com/file/d/1EwMJFjNp2kuNglutoleZOVsqccSOW2Z4/view?usp=sharing) |
| 视频超分 | 链接: https://pan.baidu.com/s/131Ul2A9DNxTXbatO1SGKFg?pwd=viy5 提取码: viy5 | [google](https://drive.google.com/file/d/1oyrVL20MODJOSf7BJ9T5OioE-ZaARDBC/view?usp=sharing) |

取最新的 models_xxx.zip，解压、软链为 examples/models

```bash
$ wget  ${URL}/modes.zip
$ cd flow-python/examples/application
$ ln -s ${DOWNLOAD_DIR}/models models
```

如果有 MegFlow-models repo，可以直接

```bash
$ cd MegFlow-models
$ git-lfs update
$ git lfs pull
```
