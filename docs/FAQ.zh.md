# FAQ

Q：`cargo run --example run_with_plugins -- -p logical_test ` 无法运行怎么办？

A1：如果报错 `message: "No such file or directory" }'`，确认是否`cd flow-python/examples`

A2：确认安装了 python 组件，即 `cd flow-python` 再 `python3 setup.py install --user`

还不行就提 issue。

___
Q：视频流跑一路没事。跑多个，内存爆了/显存爆了/模型加载多次是因为啥？

A：参照 `cat_finder/video.toml`，把涉模型的 nodes 移到 `[[graphs]]`上面，让多个子图来共享。

每启动一个视频流就会启一个子图，如果`nodes`放到`[[graphs]]`里，20 路视频就会创建 20 套 nodes。
___
Q：如何修改服务端口号，8080 已被占用？

A：以`cat_finder` 为例，端口配置在 `image.toml` 的 `port` 中。
___
Q：如何让 ImageServer 返回 json，而不是渲染后的图？

A：ImageServer 默认返回 `envelope.msg["data"]`图像。如果要返回 json 需要改两处：
* `image.toml` 的配置里增加 `response = "json"`
* 最终结果用`self.out.send(envelope.repack(json.dumps(results)))`发送出去

框架既不知道哪些字段可序列化，也不知道要序列化那几个字段，因此需要调用方序列化成 `str`。代码可参照 `examaples/cat_finder/redis_proxy.py`。
___
Q：视频流为什么无法停止，调用了 stop 接口还是在处理？

A：调用 stop 之后队列的 `push/put` 接口已经被关闭了，不能追加新的，但之前解好的帧还在队列里。需要把遗留的处理完、依次停止子图节点才完全结束。流不会调用 stop 即刻停止，实际上有延迟。
___
