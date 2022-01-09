#!/usr/bin/env python
# coding=utf-8

from megflow import block_on, join, Graph, Envelope
from megflow.func_op import map_def

@map_def()
def noop(x):
    return x

if __name__ == '__main__':
    g = Graph(config_str="""
main = "noop"
[[graphs]]
name = "noop"
inputs = [
    { name = "inp", cap = 16, ports=["n:inp"] },
]
outputs = [
    { name = "out", cap = 16, ports=["n:out"] },
]
    [[graphs.nodes]]
    name = "n"
    ty = "Noop"
              """)

    n = 160
    def send():
        inp = g.input('inp')
        for i in range(n):
            inp.send(Envelope.pack(i))
        g.close()

    def recv():
        out = g.output('out')
        recv_n = 0
        while True:
            envelope = out.recv()
            if envelope is None:
                break
            recv_n = envelope.msg
        assert recv_n == n - 1

    block_on(lambda: join([recv, send]))
    g.wait()
    print('finish')

