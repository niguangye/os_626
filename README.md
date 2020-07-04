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
target = "x86_64-os_626.json"
```



### 2.5 向屏幕打印字符

我们预先定义了一个**字节串**（byte string）类型的**静态变量**（static variable），名为`HELLO`。

我们首先将整数`0xb8000`**转换**（cast）为一个**裸指针**（[raw pointer](https://doc.rust-lang.org/stable/book/second-edition/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer)）。

这之后，我们迭代`HELLO`的每个字节，使用[enumerate](https://doc.rust-lang.org/core/iter/trait.Iterator.html#method.enumerate)获得一个额外的序号变量`i`。

在`for`语句的循环体中，我们使用[offset](https://doc.rust-lang.org/std/primitive.pointer.html#method.offset)偏移裸指针，解引用它，来将字符串的每个字节和对应的颜色字节——`0xb`代表淡青色——写入内存位置。

### 2.6 启动内核

- 引入bootloader包

- 安装bootimage工具

-  编译镜像

  ```
  # 编译镜像
  cargo bootimage
  1. 编译我们的内核为一个ELF（Executable and Linkable Format）文件；
  2. 编译引导程序为独立的可执行文件；
  3. 将内核ELF文件按字节拼接（append by bytes）到引导程序的末端。
  ```

- 在QEMU中启动内核

  >  当机器启动时，引导程序将会读取并解析拼接在其后的ELF文件。这之后，它将把程序片段映射到**分页表**（page table）中的**虚拟地址**（virtual address），清零**BSS段**（BSS segment），还将创建一个栈。最终它将读取**入口点地址**（entry point address）——我们程序中`_start`函数的位置——并跳转到这个位置。

  ```
   qemu-system-x86_64 -drive format=raw,file=bootimage-blog_os.bin
  ```

- 使用cargo run

  ```
  #在cargo配置文件中设置`runner`配置项
  # in .cargo/config
  
  [target.'cfg(target_os = "none")']
  runner = "bootimage runner"
  
  # 启动
  cargo xrun
  ```

## 三、VGA文本模式

VGA字符缓冲区是一个25行、80列的二维数组，它的内容将被实时渲染到屏幕。这个数组的元素被称作**字符单元**（character cell）

| Bit(s) | Value            |
| ------ | ---------------- |
| 0-7    | ASCII code point |
| 8-11   | Foreground color |
| 12-14  | Background color |
| 15     | Blink            |

### 3.1 包装到Rust模块

```
# 引入vga_buffer模块

// in src/main.rs
mod vga_buffer;
```

### 3.2 字符缓冲区

### 3.3 打印字符

```
# 这个函数首先创建一个指向0xb8000地址VGA缓冲区的Writer。
# 实现这一点，我们需要编写的代码可能看起来有点奇怪：
# 首先，我们把整数0xb8000强制转换为一个可变的裸指针（raw pointer）；
# 之后，通过运算符*，我们将这个裸指针解引用；最后，我们再通过&mut，再次获得它的可变借用。
# 这些转换需要**unsafe语句块**（unsafe block），因为编译器并不能保证这个裸指针是有效的。
let mut writer = Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
};

```

### 3.4 易失操作

### 3.5 格式化宏

支持Rust提供的**格式化宏**（formatting macros）也是一个相当棒的主意。

通过这种途径，我们可以轻松地打印不同类型的变量，如整数或浮点数。

为了支持它们，我们需要实现[`core::fmt::Write`](https://doc.rust-lang.org/nightly/core/fmt/trait.Write.html) trait；

要实现它，唯一需要提供的方法是`write_str`，它和我们先前编写的`write_string`方法差别不大，只是返回值类型变成了`fmt::Result`：

### 3.6 实现换行等函数

### 3.7 全局接口

### 3.8 延迟初始化

### 3.9 自旋锁

### 3.10 println!宏



## 致谢

- 感谢 @[phil-opp ](https://github.com/phil-opp) 的[blog_os项目](https://github.com/phil-opp/blog_os)，[开发博客](https://os.phil-opp.com/)。

- 感谢 [@luojia65](https://github.com/luojia65)同学由[blog_os项目](https://github.com/phil-opp/blog_os)译制的 [使用Rust编写操作系统](https://github.com/rustcc/writing-an-os-in-rust) 一书。

- 感谢 [rCore-Tutorial V3](https://rcore-os.github.io/rCore-Tutorial-deploy/) 文档的编写者。



