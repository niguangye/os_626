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

- **缺页错误（Page Fault）**：

- **非法操作码（Invalid Opcode）**：
- **通用保护错误（General Protection Fault）**：
- **双重异常（Double Fault）**：
- **三重异常（Triple Fault）**：

你可以在[这里](https://wiki.osdev.org/Exceptions)找到所有的CPU异常列表。

### 中断描述符表（interrupt descriptor table)

为了捕获并处理CPU异常，我们需要建立所谓的中断描述符表（interrupt descriptor table，IDT)。在IDT中，我们可以为每种异常指定一个处理函数。硬件会直接使用这张表，所以我们需要遵循提前约定好的格式。IDT的每一项（entry）必须是16字节的结构：

| Type | Name             | Description                                                  |
| ---- | ---------------- | ------------------------------------------------------------ |
| u16  | 函数指针 [0:15]  | 处理函数（handler function)指针的低16位                      |
| u16  | GDT 选择子       | [global descriptor table](https://en.wikipedia.org/wiki/Global_Descriptor_Table) 代码段的选择子 |
| u16  | 选项参数         | 参见下文                                                     |
| u16  | 函数指针 [16:31] | 处理函数（handler function)指针的中间16位                    |
| u32  | 函数指针 [32:63] | 处理函数（handler function)指针剩下的32位                    |
| u32  | 保留位           |                                                              |

选项参数必须是下面的结构：

| Bits  | Name                 | Description                                                  |
| ----- | -------------------- | ------------------------------------------------------------ |
| 0-2   | 中断栈表索引         | 0: 不切换栈, 1-7:当处理函数被调用时，切换到中断栈表（Interrupt Stack Table）的第n个栈 |
| 3-7   | 保留位               |                                                              |
| 8     | 0: 中断门, 1: 陷阱门 | 如果这个bit被设置为0，处理函数被调用的时候，中断会被禁用。   |
| 9-11  | 必须为1              |                                                              |
| 12    | 必须为0              |                                                              |
| 13‑14 | 特权等级描述符 (DPL) | 允许调用该处理函数的最小特权等级。                           |
| 15    | Present              |                                                              |

每个异常都拥有提前约定好的IDT索引。例如：非法操作码的表索引是6，而缺页错误的的表索引是14。因此，硬件可以找到每种异常对应的中断描述符表的条目（interrupt descriptor table entry, IDT entry)。[OSDev wiki](https://wiki.osdev.org/Exceptions)页面的Exception Table的“Vector nr.”列展示了所有异常的IDT索引。

当异常发生时，CPU大致遵循下面的流程：

1. 将一些寄存器的内容压入栈中，包括当前指令的指针和[RFLAGS](http://en.wikipedia.org/wiki/FLAGS_register)寄存器的内容（我们会在文章的后续部分用到这些值）。

2. 读取中断描述符表（IDT）中对应的条目。例如：缺页错误发生时，CPU会读取IDT的第十四个条目。
3. 检查这个条目是否存在，如果没有则升级为双重错误（double fault)。
4. 如果条目是一个中断门（第40个bit没有被设置为1），则禁用硬件中断。
5. 装载指定的GDT 选择子到CS段。
6. 跳转到指定的处理函数。

现在不要担心第四、五步，我们会在未来的文章中研究GDT和硬件中断。