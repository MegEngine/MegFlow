# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
import cv2
import requests


def test():
    ip = 'localhost'
    port = '8084'
    user_define_string = 'content'
    url = f'http://{ip}:{port}/analyze/{user_define_string}'
    img = cv2.imread("./test.jpg")
    _, data = cv2.imencode(".jpg", img)
    data = data.tobytes()

    headers = {'Content-Length': f'{len(data)}', 'Content-Type': 'image/*'}
    res = requests.post(url, data=data, headers=headers)
    print(res.content)


if __name__ == "__main__":
    test()
