# /bin/python3

import os
import re
import sys
import argparse
import requests

def make_parser():
    parser = argparse.ArgumentParser("Doc link checker")
    parser.add_argument("--http",
                        default=False,
                        type=bool,
                        help="check http or not ")
    return parser

def accessible(url):
    resp = requests.get(url)
    return resp.ok

pattern = re.compile(r'\[.*?\]\(.*?\)')
def analyze_doc(home, path, args):
    problem_list = []
    with open(path) as f:
        lines = f.readlines()
        for line in lines:
            if '[' in line and ']' in line and '(' in line and ')' in line:
                all = pattern.findall(line)
                for item in all:
                    start = item.find('(')
                    end = item.find(')')
                    ref = item[start+1: end]
                    if ref.endswith('.py') or ref.endswith('.rs'):
                        if not ref.startswith('http'):
                            problem_list.append(ref)
                            continue
                    if ref.startswith('http') or ref.startswith('#'):
                        if args.http == True and ref.startswith('http') and 'github' in ref and 'megengine' in ref.lower():
                            if accessible(ref) == False:
                                problem_list.append(ref)
                        continue
                    fullpath = os.path.join(home, ref)
                    if not os.path.exists(fullpath):
                        problem_list.append(ref)
                        # print(f' {fullpath}  in  {path} not exist!')
            else:
                continue
    if len(problem_list) > 0:
        print(f'{path}:')
        for item in problem_list:
            print(f'\t {item}')
        print('\n')


def traverse(_dir, args):
    for home, dirs, files in os.walk(_dir):
        if "./target" in home or "./.github" in home:
            continue
        for filename in files:
            if filename.endswith('.md'):
                path = os.path.join(home, filename)
                if os.path.islink(path) == False:
                    analyze_doc(home, path, args)


if __name__ == "__main__":
    args = make_parser().parse_args()
    traverse(".", args)
