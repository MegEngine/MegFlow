# upgrade pip
python3 -m pip install --upgrade pip
# install partial requirements with tuna mirror
python3 -m pip install -i https://pypi.tuna.tsinghua.edu.cn/simple -r requires.txt
# install megenginelite for inference
python3 -m pip install megengine -f https://megengine.org.cn/whl/mge.html

