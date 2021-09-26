# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
import urllib
import requests


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
