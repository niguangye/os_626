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

### 内核栈溢出



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



