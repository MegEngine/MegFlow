import sys
import os
import requests

def main(ip, filename):
    url_template = "http://{}:8090/control/get?room=megflow-test"
    url = url_template.format(ip)
    print(url)
    result = requests.get(url)
    if not result.ok:
        raise Exception('req channel failed')
    channel = result.json()['data']
    upload_cmd = f'ffmpeg -re -i {filename}  -c copy -f flv rtmp://{ip}:1935/live/{channel}'
    # upload flv
    os.system(upload_cmd)


if __name__ == "__main__":
    videoname = 'demo.flv'
    if len(sys.argv) < 2:
        print(f'usage python3 {sys.argv[0]} videoname, use default demo.flv')
    else:
        videoname = sys.argv[1]
    main('localhost', videoname)
