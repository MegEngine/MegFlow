# /bin/python3

import os
import re
import sys

pattern = re.compile(r'\[.*?\]\(.*?\)')
def analyze_doc(home, path):
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
                    if ref.startswith('http') or ref.startswith('#'):
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
        sys.exit(1)

def traverse(_dir):
    for home, dirs, files in os.walk(_dir):
        if "./target" in home or "./.github" in home:
            continue
        for filename in files:
            if filename.endswith('.md'):
                path = os.path.join(home, filename)
                if os.path.islink(path) == False:
                    analyze_doc(home, path)

if __name__ == "__main__":
    traverse(".")
