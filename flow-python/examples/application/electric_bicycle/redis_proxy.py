# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import redis
from loguru import logger
from megflow import register


@register(inputs=['inp'], outputs=['out'])
class RedisProxy:
    def __init__(self, name, args):
        logger.info("init redis pool...")
        self.name = name
        self._key = args['key']
        self._db = dict()

        ip = args['ip']
        port = args['port']

        self._pool = redis.ConnectionPool(host=ip,
                                            port=port,
                                            decode_responses=False)
        logger.info('redis pool initialized.')


    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return
        msg = envelope.msg
        crops = msg['shaper']
        assert isinstance(crops, list)

        if len(crops) > 0:
            try:
                r = redis.Redis(connection_pool=self._pool)
                alert = f'{len(crops)}  electric bicycle occur !'
                logger.info('alert {}'.format(alert))
                r.lpush(self._key, alert)
                self.out.send(envelope.repack(alert))
            except redis.exceptions.ConnectionError as e:
                logger.error(str(e))
