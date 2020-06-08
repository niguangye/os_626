# 使用Rust编写操作系统

使用Rust编写操作系统，屏蔽掉过于底层的硬件细节和汇编语言，这有助于将主要精力放在内存管理、进程管理、文件系统等核心模块。

对操作系统启动之前的主引导，跳入保护模式，加载内核等过程有兴趣的可以参考[reduced-os（已封存）](https://github.com/niguangye/reduced-os)

## 一、不依赖标准库的Rust可执行程序

> 要编写一个操作系统内核，我们的代码应当不基于任何的操作系统特性。这意味着我们不能使用线程、文件、堆内存、网络、随机数、标准输出，或其它任何需要特定硬件和操作系统抽象的特性。
>
> 这意味着我们不能使用[Rust标准库](https://doc.rust-lang.org/std/)的大部分。
>
> 但我们可以使用[迭代器](https://doc.rust-lang.org/book/ch13-02-iterators.html)、[闭包](https://doc.rust-lang.org/book/ch13-01-closures.html)、[模式匹配](https://doc.rust-lang.org/book/ch06-00-enums.html)、[Option](https://doc.rust-lang.org/core/option/)、[Result](https://doc.rust-lang.org/core/result/index.html)、[字符串格式化](https://doc.rust-lang.org/core/macro.write.html)，当然还有[所有权系统](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)。

### 1.1 禁用标准库



### 1.2.实现panic处理函数



### 1.3 重写入口点



### 1.4 链接器错误



## 致谢

- 感谢 @[phil-opp ](https://github.com/phil-opp) 的[blog_os项目](https://github.com/phil-opp/blog_os)，[开发博客](https://os.phil-opp.com/)。

- 感谢 [@luojia65](https://github.com/luojia65)同学由[blog_os项目](https://github.com/phil-opp/blog_os)译制的 [使用Rust编写操作系统](https://github.com/rustcc/writing-an-os-in-rust) 一书。

- 感谢 [rCore-Tutorial V3](https://rcore-os.github.io/rCore-Tutorial-deploy/) 文档的编写者。



