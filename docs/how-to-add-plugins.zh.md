# Python Plugins

从一个最简单的例子开始
```
import megflow
@megflow.register(name="alias", inputs=["inp"], outputs=["out"])
class Node:
    def __init__(self, name, args):
        pass
    def exec(self):
        envelope = self.inp.recv()
        msg = dowith(envelope.msg)
        self.out.send(envelope.repack(msg))
```

这其中有三部分内容: register装饰器，Node的构造函数，Node的执行函数

1. register装饰器
    - name: 若指定，则register所修饰插件重命名为name，默认为register所修饰类的类名
    - inputs: Node的输入列表，每个输入`input`都可以在`exec`方法中，通过`self.input`访问,
    - outputs: Node的输出列表，每个输出`output`都可以在`exec`方法中，通过`self.output`访问
    - exclusive: 默认为False, 调度模型是一个thread local的协程调度器, 若为True, 则将该任务安排到线程池中

2. Node的构造函数
    - name: 即参数文件中Node的name字段
    - args: 即参数文件中Node的剩余参数字段

3. Node的执行函数
    - 一个python插件的执行方法必须是命名为`exec`的零参成员方法
    - 对于在参数文件中该插件引用的资源`resource`, 可以在`exec`方法中，通过`self.resource`访问
    - 通过输入的`recv`方法取得输入消息，输入消息是`Envelope`对象，其`msg`成员即开发者真正需要读写的消息实例
    - `Envelope`语义为在图中流转的消息的相关信息，由于这些信息需要在图中被传递，所以开发者应该保持消息与`Envelope`的对应关系
    - 若一个`Envelope`携带的消息被拆分为多个消息，或者转换为另一个消息，应该通过`Envelope`的`repack`方法，将`Envelope`与消息关联起来
    - 通过输出的`send`方法发送输出消息，输出消息是`Envelope`对象

MegFlow也提供了一系列异步工具
1. `yield_now()`, 让出当前任务的执行权
2. `sleep(dur)`, 使当前任务沉睡`dur`毫秒
3. `join(tasks)`, `tasks`参数是一个函数列表，`join`堵塞直到`tasks`中的函数都执行完毕
4. `create_future(callback)`, `callback`参数是一个函数, 默认值为None，`create_future`返回一个`(Future, Waker)`对象
    - `Future::wait`, 堵塞直到`Waker::wake`被调用，返回`Waker::wake(result)`传入的`result`参数

# Rust Plugins

Coming soon
