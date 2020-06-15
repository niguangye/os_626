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

```
#![no_std] // 禁用标准库链接
```

### 1.2.实现panic处理函数

在`no_std`环境中，我们需要定义自己的panic处理函数。

禁用栈展开

### 1.3 重写入口点

一个典型的使用标准库的Rust程序，它的运行将从名为`crt0`的运行时库开始。`crt0`意为C runtime zero，它能建立一个适合运行C语言程序的环境，这包含了栈的创建和可执行程序参数的传入。这之后，这个运行时库会调用[Rust的运行时入口点](https://github.com/rust-lang/rust/blob/bb4d1491466d8239a7a5fd68bd605e3276e97afb/src/libstd/rt.rs#L32-L73)，这个入口点被称作**start语言项**（"start" language item）。

这之后，运行时将会调用main函数。

我们的独立式可执行程序并不能访问Rust运行时或`crt0`库，所以我们需要定义自己的入口点。

### 1.4 链接器错误

链接器的默认配置假定程序依赖于C语言的运行时环境，但我们的程序并不依赖于它。

为了解决这个错误，我们需要告诉链接器，它不应该包含（include）C语言运行环境。

- 可以提供特定的**链接器参数**（linker argument），

- 也可以选择编译为**裸机目标**（bare metal target），即底层没有操作系统的运行环境。

  > ```
  > # 安装thumbv7em-none-eabihf裸机环境，一个ARM嵌入式系统
  > rustup target add thumbv7em-none-eabihf
  > # 编译
  > cargo build --target thumbv7em-none-eabihf
  > ```

## 二、最小化内核

> 我们将向显示器打印字符串，最终打包内核为能引导启动的**磁盘映像**（disk image）

### 2.1 引导启动

- BIOS启动
- Multiboot标准

### 2.2 最小化内核

在默认情况下，`cargo`会为特定的**宿主系统**（host system）构建源码，比如为你正在运行的系统构建源码。

这并不是我们想要的，因为我们的内核不应该基于另一个操作系统——我们想要编写的，就是这个操作系统。

确切地说，我们想要的是，编译为一个特定的**目标系统**（target system）

### 2.3 安装Nightly Rust

Rust语言有三个**发行频道**（release channel），分别是stable、beta和nightly.

```
# 使用rustup安装nightly版本
rustup override add nightly
# 查看rust版本
rustc --version
```

### 2.4 编译内核

目标配置清单：只需使用一个JSON文件，Rust便允许我们定义自己的目标系统。

```
# 安装Cargo xbuild
cargo install cargo-xbuild
# 编译
cargo xbuild --target x86_64-os_626.json
```

设置默认目标，避免每次使用`cargo xbuild`传递参数

```
# in .cargo/config

[build]
target = "x86_64-blog_os.json"
```



### 2.5 向屏幕打印字符

### 2.6 启动内核

## 致谢

- 感谢 @[phil-opp ](https://github.com/phil-opp) 的[blog_os项目](https://github.com/phil-opp/blog_os)，[开发博客](https://os.phil-opp.com/)。

- 感谢 [@luojia65](https://github.com/luojia65)同学由[blog_os项目](https://github.com/phil-opp/blog_os)译制的 [使用Rust编写操作系统](https://github.com/rustcc/writing-an-os-in-rust) 一书。

- 感谢 [rCore-Tutorial V3](https://rcore-os.github.io/rCore-Tutorial-deploy/) 文档的编写者。



