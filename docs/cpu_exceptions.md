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

## 一个IDT类型（An IDT Type)

我们选择使用`x86_64` crate中的 `InterruptDescriptorTable` 结构体，而不是创建自己的 IDT 类型：

```
#[repr(C)]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFunc>,
    pub debug: Entry<HandlerFunc>,
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    pub overflow: Entry<HandlerFunc>,
    pub bound_range_exceeded: Entry<HandlerFunc>,
    pub invalid_opcode: Entry<HandlerFunc>,
    pub device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<HandlerFuncWithErrCode>,
    pub invalid_tss: Entry<HandlerFuncWithErrCode>,
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
    pub page_fault: Entry<PageFaultHandlerFunc>,
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncWithErrCode>,
    pub machine_check: Entry<HandlerFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    pub security_exception: Entry<HandlerFuncWithErrCode>,
    // some fields omitted
}
```

`InterruptDescriptorTable`结构体的字段都是[`idt::Entry`](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/struct.Entry.html)类型，这种类型是一种代表`IDT`条目字段的结构体（见上面的示例）。类型参数`F`定义了预期的处理函数类型。我们可以发现上面的条目字段需要 [`HandlerFunc `](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/type.HandlerFunc.html) 或 [`HandlerFuncWithErrCode `](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/type.HandlerFuncWithErrCode.html) 参数。缺页错误甚至拥有它独有的处理函数类型：[`PageFaultHandlerFunc `](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/type.PageFaultHandlerFunc.html) 。

首先，我们探讨一下 `HandlerFunc` 类型：

```
type HandlerFunc = extern "x86-interrupt" fn(_: &mut InterruptStackFrame);
```

`HandlerFunc ` 是 `extern "x86-interrupt" fn` 的类型别名。`extern` 关键字定义了一个外部调用约定（ [foreign calling convention](https://doc.rust-lang.org/nomicon/ffi.html#foreign-calling-conventions) ），它经常被用于链接C语言代码（`extern "C" fn`）。那么，`x86-interrupt`调用约定是什么呢?

## 中断调用约定（ The Interrupt Calling Convention）

CPU异常与函数调用非常相似：CPU跳转到调用函数的第一条指令并执行它。然后，CPU跳转到返回地址并继续执行函数的调用者函数（`parent function`)。

```
译者注：函数调用即A函数在执行过程中，调用了B函数，待B函数执行完毕后，回到A函数继续执行。
A函数被称为调用者函数，B函数即被调用者函数。
function A {
	...
	B();
	...
}
```

然而，异常和函数调用有一个重要的区别：函数调用是被编译器生成的 `call` 指令主动发起，而

异常可以发生在所有指令的执行过程中。为了理解这个区别的重要性，我们需要更进一步地研究函数调用。

[调用约定 Calling conventions](https://en.wikipedia.org/wiki/Calling_convention) 明确规定了函数调用的细节。例如，它规定了函数参数的位置（ 寄存器还是函数栈）和结果的返回方式。在x86_64 Linux体系中，C语言函数调用适用下面的规则（在[System V ABI](https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf)中规定）：

- 前六个整数参数会被放在寄存器中传递：`rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
- 剩下的参数被放在栈中传递
- 结果被放在 `rax` 和 `rdx` 中返回

Rust 没有遵顼C ABI （事实上，Rust甚至没有规定的ABI），所以这些规则仅仅适用于声明了 `extern "C" fn` 的函数。

###  Preserved and Scratch 寄存器

调用约定（ `calling convention`）将寄存器分为两个部分： *preserved* 和 *scratch* 寄存器。

 在函数调用的过程中，*preserved*寄存器的值必须保持不变。所以，被调用的函数（`callee`）必须保证会在返回以前会主动复原这些寄存器的原始值，才可以修改这些寄存器的值。因此，这些寄存器被称为被**调用者保存寄存器**（*callee-saved*，译者注：也就是AKA非易失性寄存器）。通行的模式是在函数的开始保存这些寄存器的值到函数栈中，并在函数马上返回的时候复原他们。

相比之下，被调用的函数（`callee`）可以无约束地修改 *scratch*寄存器。如果调用者函数希望在函数调用的过程中保留 *scratch*寄存器的值，它需要在调用函数之前备份和复原 *scratch*寄存器的值（例如将这些值压入栈中）。所以，这些寄存器被称为**调用者寄存器**（*caller-saved*，译者注：也就是AKA易失性寄存器）。

 在x86_64架构中，C语言调用约定明确规定了下面的 preserved and scratch 寄存器：

| preserved 寄存器                                | scratch 寄存器                                              |
| ----------------------------------------------- | ----------------------------------------------------------- |
| `rbp`, `rbx`, `rsp`, `r12`, `r13`, `r14`, `r15` | `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`, `r9`, `r10`, `r11` |
| *callee-saved*                                  | *caller-saved*                                              |

编译器遵顼这些规定生成二进制字节码。例如：绝大多数函数地字节码开始于`push rbp`指令，这个指令会备份`rbp`寄存器地值到函数栈中（因为这是一个`callee-saved`寄存器）。

### 保存所有寄存器

与函数调用形成鲜明对比的是，异常可以发生在所有指令的执行过程中。大多数情况下，我们甚至不能识别出编译器生成的代码是否会引起异常。例如，编译器不能预见到一个指令是否会引起栈溢出或缺页错误。

既然不能预见到异常的发生时机，我们自然也无法做到提前备份任何寄存器的值。这意味着我们不能使用依赖于 `caller-saved` 寄存器的调用约定去处理异常。然而，我们需要一个会保存所有寄存器值的调用约定。`x86-interrupt`调用约定恰恰能够保证所有寄存器会在函数调用结束以前复原到原始值。

这并不意味着所有寄存器的值会在函数开始时被保存到函数栈中。相反，编译器（生成的代码）只会备份被函数覆盖的寄存器的值。在这种方式下，较短的函数编译生成的二进制字节码会非常高效，也就是只使用尽可能少的寄存器。

### 中断栈帧（ The Interrupt Stack Frame）

在寻常的函数调用（`call`指令执行）中，CPU跳转到相应的函数之前会将返回地址压入到函数栈中。在函数返回（`ret`指令执行）的时候，CPU会弹出并跳转到这个返回地址。所以，寻常的函数调用栈帧会如下图所示：

![function-stack-frame](C:\Users\Administrator\Downloads\function-stack-frame.svg)

然而，异常和中断处理函数并不能将返回地址压入到函数栈中，因为中断处理函数往往运行在不同的上下文（栈指针，CPU flags等）中。相反，在异常发生的时候，CPU会执行以下步骤：

1. **对齐栈指针**：中断可以发生在任何指令的执行过程中，栈指针自然也可能是任何值。然而，一些CPU指令集（e.g. 一些 SSE指令集）需要栈指针在16字节边界上对齐，因此CPU会在中断之后靠右对齐栈指针。
2. **切换栈（在某种情况下）**：CPU特权等级发生改变的时候，栈会被切换，例如CPU 异常发生在用户态程序的时候。用所谓的中断栈表（ *Interrupt Stack Table* , 下篇文章解释 ）配置特定中断的栈切换也是可行的。
3. **压入原来的栈指针**：在中断发生的时候（对齐栈指针发生之前），CPU将栈指针（`rsp`）和栈段（`ss`)寄存器压入栈中。如此一来，中断处理函数返回时就可以复原栈指针的原始值。
4. **压入并更新`RFLAGS`寄存器**：[`RFLAGS`](https://en.wikipedia.org/wiki/FLAGS_register)寄存器保存了多种控制和状态位。进入中断函数时，CPU修改一些位并压入旧的值。
5. **压入指令指针**：跳转到中断处理函数之前，CPU压入指令指针（`rip`）和代码段（`cs`)。这类似于寻常的函数调用压入返回地址的过程。
6. **压入错误码（对于部分异常）**：对于缺页错误等特定的异常，CPU会压入解释异常原因的错误码。
7. **调用中断处理函数**：CPU从IDT对应的字段中读取中断处理函数的地址和段描述符。然后通过加载这些值到`rip`和`cs`寄存器中，调用中断处理函数。

所以，中断调用栈帧会如下图所示：

![exception-stack-frame](https://markdown-ngy.oss-cn-beijing.aliyuncs.com/exception-stack-frame.svg)

在Rust的`x86_64`库中，中断调用栈帧被抽象为[`InterruptStackFrame`](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/struct.InterruptStackFrame.html)结构体。它会被作为`&mut`传递给中断处理函数，并被用来获取更多的关于异常原因的信息。由于只有小部分异常会压入错误码，所以[`InterruptStackFrame`](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/struct.InterruptStackFrame.html)并没有设置`error_code`字段。这些异常会另外使用[`HandlerFuncWithErrCode`](https://docs.rs/x86_64/0.12.1/x86_64/structures/idt/type.HandlerFuncWithErrCode.html)函数来处理，这个函数有一个`error_code`参数用来保存错误码。

### 幕后工作

`x86-interrupt`调用约定作为一个优秀的抽象，它几乎隐藏了异常处理过程中所有繁杂的细节。然而，理解幕布后的工作在某些时候是有益的。下面简要概述了`x86-interrupt`调用约定所处理的事情：

- **抽取参数**：大多数调用约定希望参数被放在寄存器中传递。这对于异常处理函数是不可能的，因为我们不能在保存寄存器的值之前覆盖这些寄存器。然而，`x86-interrupt`调用约定明白这些参数早就被放在栈的某个位置上了。
- **使用`iretq`返回**：既然中断栈帧和寻常函数调用的栈帧是不同的，我们不能使用`ret`指令从中断处理函数中返回。但是可以使用`iretq`指令。
- **处理错误码**：部分特定异常压入的错误码是事情变得更加复杂。它改变了栈对齐（见对齐栈部分）并且需要在返回之前从栈中弹出。`x86-interrupt`调用约定处理了所有难题。但是，它无法获得每种异常对应的处理函数，所以，它需要从函数的参数中推断这些信息。这意味着，程序员有责任使用正确的函数类型处理每种异常。幸运的是，`x86_64`库的`InterruptDescriptorTable`可以确保这一过程不会出错。
- **对齐栈**：

如果你对更多的细节感兴趣：我们也有一系列文章解释了如何使用 [naked functions](https://github.com/rust-lang/rfcs/blob/master/text/1201-naked-fns.md)处理异常。

