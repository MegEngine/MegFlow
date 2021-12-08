#!/usr/bin/env python
# coding=utf-8
import argparse

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='generate config.toml for profile.')
    parser.add_argument('-n', '--node_num', type=int, default=1, help='node num')
    parser.add_argument('-d', '--data_num', type=int, default=1, help='data num')

    args = parser.parse_args()

    node_num = args.node_num
    data_num = args.data_num

    header = '''main = "Example"
[[graphs]]
name = "Example"

[[graphs.connections]]
cap = 64
ports = ["src:out", "trans0:inp"]
[[graphs.connections]]
cap = 64
ports = ["sink:inp", "trans{}:out"]

'''.format(node_num-1)

    def build_node(i):
        return '''[[graphs.nodes]]
name = "trans{}"
ty = "Transport"
'''.format(i)

    def build_conn(i):
        return '''[[graphs.connections]]
cap = 64
ports = ["trans{}:inp", "trans{}:out"]
'''.format(i+1, i)
    
    nodes = []
    conns = []
    for i in range(node_num):
        nodes.append(build_node(i))
    for i in range(node_num - 1):
        conns.append(build_conn(i))

    config = header + ''.join(conns) + ''.join(nodes) + '''
[[graphs.nodes]]
name = "src"
ty = "Source"
n = {}
[[graphs.nodes]]
name = "sink"
ty = "Sink"
n = {}
    '''.format(data_num, data_num)

    print(config)
