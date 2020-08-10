# CPU异常

> 原文：[https://os.phil-opp.com/cpu-exceptions/](https://os.phil-opp.com/cpu-exceptions/)
>
> 原作者：@phil-opp
>
> 译者：[倪广野](https://github.com/niguangye)

触发CPU异常的情况多种多样，例如：访问非法内存地址或执行非法指令（除以零）等。为了应对CPU异常，我们需要建立中断描述符表（interrupt descriptor table)，它列举了不同异常所对应的处理函数(handler functions)。在博文的最后，我们的内核（kernel)可以捕获断点异常（[breakpoint exceptions](https://wiki.osdev.org/Exceptions#Breakpoint)）并且恢复CPU的正常运行。

## 概述

异常的发生标志着当前正在执行的指令出现了问题。例如：指令试图除以0的时候，CPU会抛出一个异常。当异常发生，CPU会中断（interrupt）它当前的流程，并立即调用该类型异常对应的处理函数。

> 译注：对应的处理函数需要在中断描述符表（interrupt descriptor table)中查找。

在x86体系结构中，有大约20种不同的CPU 异常类型。常见的如下：

- **缺页异常（Page Fault）**：

- **非法操作码（Invalid Opcode）**：
- **通用保护错误（General Protection Fault）**：
- **双重异常（Double Fault）**：
- **三重异常（Triple Fault）**：

你可以在[这里](https://wiki.osdev.org/Exceptions)找到所有的CPU异常列表。

### 中断描述符表（interrupt descriptor table)