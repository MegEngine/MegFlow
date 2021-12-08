# Megvii Electric Moped Detector
包含电动车检测 ONNX 模型及推理代码

<img src="docs/_static/elec_cycle_demo_1.gif" alt="gif1" height="320"> <img src="docs/_static/elec_cycle_demo_2.gif" alt="gif2" height="320"> <img src="docs/_static/elec_cycle_demo_3.gif" alt="gif3" height="320">

完整视频: [demo_1](https://v.qq.com/x/page/k3257ewacxa.html), [demo_2](https://v.qq.com/x/page/e3257kxhhut.html), [demo_3](https://v.qq.com/x/page/y32572ztgnt.html)

## 准备工作
安装 cuda 10.1 以及 cudnn 7.6.3

## 安装
```
git clone https://github.com/megvii-research/MEMD.git
cd MEMD
pip3 install -r requirements.txt
```

## 推理
```
python3 scripts/inference.py  --detector ./models/model.onnx --input-img ./demo/input.jpg --model-json ./models/config.json --output-path ./demo/output.jpg

--detector     ：onnx模型
--input-img    ：输入图片地址
--model-json   ：模型的配置文件地址
--output-path  ：检测结果文件地址
```
