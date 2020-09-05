# 双重异常

> 原文：[https://os.phil-opp.com/double-fault-exceptions/](https://os.phil-opp.com/double-fault-exceptions/)
>
> 原作者：@phil-opp
>
> 译者：[倪广野](https://github.com/niguangye)

这篇文章将深入探究双重异常（*double fault*），这是一个在CPU调用异常处理函数失败的时候触发的异常。通过处理双重异常，可以避免会引起系统复位的三重异常。为了彻底防止各种情况下的三重异常，需要建立中断栈表（ *Interrupt Stack Table* ）去捕获所有不同内核栈的双重异常。

这个博客在 [GitHub](https://github.com/phil-opp/blog_os) 上开源。如果你遇到问题或困难，请到那里提 issue 。或者你也可以在博客的最下方留言。你可以在 [`post-06`](https://github.com/phil-opp/blog_os/tree/post-06) 分支找到这篇文章的完整源码。

> 译注：中文版请移步[《编写 Rust 语言的操作系统》](https://github.com/rustcc/writing-an-os-in-rust)

## 双重异常的定义

简单点说，双重异常就是一个在CPU调用异常处理函数失败的时候触发的特定异常。例如，CPU触发缺页异常（*page fault*），但是中断描述符表（ *[Interrupt Descriptor Table](https://os.phil-opp.com/cpu-exceptions/#the-interrupt-descriptor-table)* ，*IDT*）中却没有对应处理函数的情况。所以，这和编程语言中捕获所有异常的代码块（*catch-all blocks*）有些相似，例如 C++ 中的 `catch(...)` 或 Java和 C# 中的 `catch(Exception e)` 。

双重异常的表现和普通异常区别不大。它拥有一个特定的向量号（*Interrupt Vector Number*） `8` ，我们可以在 *IDT* 中定义一个对应的处理函数。定义双重异常的处理函数十分重要，因为双重异常在不被处理的情况下会引发致命的三重异常。三重异常不能被捕获，而且会引起大多数硬件的系统复位。

### 触发一个双重异常

让我们通过触发一个没有处理函数的普通异常来引发双重异常：

```rust
// in src/main.rs

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    // trigger a page fault
    unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    };

    // as before
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}
```

我们使用 `unsafe`  去写入非法的内存地址 `0xdeadbeef` 。这个虚拟地址没有在页表中被映射到物理地址，这会触发一个缺页异常。而缺页异常的处理函数还没有被定义到 [IDT](https://os.phil-opp.com/cpu-exceptions/#the-interrupt-descriptor-table) ，因此双重异常被触发了。

现在启动内核，它会进入到无穷尽的启动循环。原因如下：

1. *CPU* 试图写入非法的内存地址 `0xdeadbeef` ，这会触发缺页异常。
2. *CPU* 查找到 *IDT* 中缺页异常对应的条目，并且没有发现对应的处理函数。因为它不能正常调用缺页异常的处理函数，所以触发了双重异常。
3. *CPU* 查找到 *IDT* 中双重异常对应的条目，并且也没有发现对应的处理函数。因此，三重异常被触发。
4. 三重异常是致命的。*QEMU* 像大多数的硬件一样选择系统复位。

所以为了阻止三重异常，我们需要提供缺页异常或双重异常的处理函数。我们希望阻止所有情况下的三重异常，因此我们选择建立所有异常未被处理时都会调用的双重异常处理函数。

## 双重异常处理函数

双重异常由普通异常和错误码组成，所以我们可以像断点异常处理函数那样定义一个双重异常处理函数。

```rust
// in src/interrupts.rs

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler); // new
        idt
    };
}

// new
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
```

双重异常处理函数打印了一个简短的错误消息和异常栈帧信息。双重异常的错误码通常会是0，所以没有必要打印出来。双重异常处理函数和断点异常处理函数的区别在于，它是一个发散函数（ [*diverging*](https://doc.rust-lang.org/stable/rust-by-example/fn/diverging.html)）。因为 `x86_64` 体系架构不允许从双重异常中返回。

现在启动内核，我们可以看见双重异常处理函数被调用了：

![qemu-catch-double-fault](https://markdown-ngy.oss-cn-beijing.aliyuncs.com/qemu-catch-double-fault.png)

工作正常！这次发生了什么：

1. *CPU* 试图写入非法的内存地址 `0xdeadbeef` ，这会触发缺页异常。
2. 像上次一样，*CPU* 查找到 *IDT* 中缺页异常对应的条目，并且没有发现对应的处理函数。因为它不能正常调用缺页异常的处理函数，所以触发了双重异常。
3. *CPU* 跳转到双重异常处理函数——它现在是就绪的了。

因为 *CPU* 现在可以正常调用双重异常处理函数，所以三重异常（和启动循环）不会再次出现。

这非常容易理解！那么我们为什么需要用整篇文章讨论这个话题? 我们现在可以捕获大多数双重异常，但是在某些情况下，现在的方式并不足够有效。



## 双重异常的触发原因

在探究某个特定的原因之前，我们需要理解双重异常的确切定义。上文中，我们给出了相当粗略的定义：

> 双重异常就是一个在CPU调用异常处理函数失败的时候触发的特定异常。

“调用异常处理函数失败”的准确含义是什么? 处理函数不可用? 处理函数被换出（ [swapped out](http://pages.cs.wisc.edu/~remzi/OSTEP/vm-beyondphys.pdf)）? 并且如果处理函数自身触发了异常会发生什么?

例如，下列情况会发生什么：

1. 断点异常触发，但是对应的处理函数被换出?
2. 缺页异常触发，但是缺页异常处理函数被换出?
3. 除0异常引发了断点异常，但是断点异常处理函数被换出?
4. 内核栈溢出，同时保护页（ *guard page*）被命中（*hit*）? 

幸运的是，AMD64手册（([PDF](https://www.amd.com/system/files/TechDocs/24593.pdf)）给出了明确定义（8.2.9章节）。根据手册的定义，“当第二个异常出现在先前的（第一个）异常处理函数执行期间，双重异常**可能**会被触发”。“**可能**”二字说明：只有特定的异常组合才会导致双重异常。这些组合是：

| 第一个异常                                                   | 第二个异常                                                   |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| [Divide-by-zero，除0](https://wiki.osdev.org/Exceptions#Divide-by-zero_Error),<br> [Invalid TSS，非法任务状态段](https://wiki.osdev.org/Exceptions#Invalid_TSS), <br/>[Segment Not Present，段不存在](https://wiki.osdev.org/Exceptions#Segment_Not_Present),<br/> [Stack-Segment Fault，栈段错误](https://wiki.osdev.org/Exceptions#Stack-Segment_Fault), <br/>[General Protection Fault，一般保护错误](https://wiki.osdev.org/Exceptions#General_Protection_Fault) | [Invalid TSS，非法任务状态段](https://wiki.osdev.org/Exceptions#Invalid_TSS), <br/>[Segment Not Present，段不存在](https://wiki.osdev.org/Exceptions#Segment_Not_Present), <br/>[Stack-Segment Fault，栈段错误](https://wiki.osdev.org/Exceptions#Stack-Segment_Fault), <br/>[General Protection Fault，一般保护错误](https://wiki.osdev.org/Exceptions#General_Protection_Fault) |
| [Page Fault，缺页异常](https://wiki.osdev.org/Exceptions#Page_Fault) | [Page Fault，缺页异常](https://wiki.osdev.org/Exceptions#Page_Fault),<br/> [Invalid TSS，非法任务状态段](https://wiki.osdev.org/Exceptions#Invalid_TSS), <br/>[Segment Not Present，段不存在](https://wiki.osdev.org/Exceptions#Segment_Not_Present),<br/> [Stack-Segment Fault，栈段错误](https://wiki.osdev.org/Exceptions#Stack-Segment_Fault),<br/> [General Protection Fault，一般保护错误](https://wiki.osdev.org/Exceptions#General_Protection_Fault) |

所以缺页异常紧跟除0异常不会触发双重异常（缺页异常处理函数被调用），但是一般保护错误紧跟除0异常一定会触发双重异常。

参考这张表格，可以得到上述前三个问题的答案：

1. 断点异常触发，但是对应的处理函数被换出，缺页异常会被触发，然后调用缺页异常处理函数。
2. 缺页异常触发，但是缺页异常处理函数被换出，双重异常会被触发，然后调用双重异常处理函数。
3. 除0异常引发了断点异常，CPU试图调用断点异常处理函数。如果断点异常处理函数被换出，缺页异常会被触发，然后调用缺页异常处理函数。

实际上，异常在 *IDT* 中没有对应的处理函数时会遵顼以下方案：

当异常发生时，*CPU* 试图读取对应的 *IDT* 条目。如果条目是0，说明这不是一个合法的 *IDT* 条目，一般保护错误会被触发。我们没有定义一般保护错误的处理函数，所以另一个一般保护错误被触发。根据上表，这会导致双重异常。

### 内核栈溢出

让我们开始探究第四个问题：

> 内核栈溢出，同时保护页（ *guard page*）被命中（*hit*）? 

保护页是存在栈底的特定内存页，它被用来发现栈溢出。保护页没有映射到任何物理内存页，所以访问它会导致缺页异常而不是无声无息地损坏其它内存。引导程序（*bootloader*）为内核栈建立了保护页，所以内核栈溢出会触发缺页异常。

当缺页异常发生，CPU 查找 IDT 中地缺页异常处理函数并将中断栈帧（ [interrupt stack frame](https://os.phil-opp.com/cpu-exceptions/#the-interrupt-stack-frame)）压入内核栈。然而，当前栈指针依然指向不可用地保护页。因此，第二个缺页异常被触发了，这会引发双重异常（根据上表）。

CPU 试图调用双重异常处理函数，它当然会试图压入异常栈帧。此时栈指针依然会指向保护页（因为栈溢出了），所以第三个缺页异常被触发了，紧接着三重异常和系统复位也发生了。当前的双重异常处理函数无法阻止这种情形下的三重异常。

让我们复现这个情形吧！通过调用无穷的递归函数可以轻易引发内核栈溢出：

```rust
// in src/main.rs

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // trigger a stack overflow
    stack_overflow();

    […] // test_main(), println(…), and loop {}
}
```

在QEMU中执行程序的时候，操作系统再次进入无限重启的情况：

如何阻止这个问题? 由于压入异常栈帧是CPU硬件的操作，所以我们不能干扰这一步。我们只能以某种方式让内核栈在双重异常触发的时候保持可用（不会溢出）。幸运的是，`x86_64` 架构提供了这个问题的解决方式。

## 切换栈

### 中断栈表和任务状态段（ The IST and TSS）

### 创建任务状态段

### 全局描述符表

### 最后的步骤



## 栈溢出测试

### 实现 `_start`

### 测试用IDT

### 双重异常处理函数



## 总结



## 接下来?



