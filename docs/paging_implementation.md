# 实现分页机制

> 原文：[https://os.phil-opp.com/double-fault-exceptions/](https://os.phil-opp.com/double-fault-exceptions/)
>
> 原作者： [@Philipp Oppermann](https://github.com/phil-opp) 
>
> 译者：[倪广野](https://github.com/niguangye)

本文演示了在内核中实现分页机制的方法。它首先探究了让内核可以访问页表对应物理内存帧的不同技术，然后讨论了其中的优缺点，最后实现了一个地址转换函数和创建新映射的函数。

这个博客在 [GitHub](https://github.com/phil-opp/blog_os) 上开源。如果你遇到问题或困难，请到那里提 issue 。或者你也可以在博客的最下方留言。你可以在 [`post-09`](https://github.com/phil-opp/blog_os/tree/post-09) 分支找到这篇文章的完整源码。

## 简介

## 访问页表

### 一对一映射

### 固定偏移映射

### 映射完整物理内存

### 临时映射

### 递归页表

## Bootloader支持

### 启动信息

### entry_point宏

## 实现

### 访问页表

### 转换地址

### 使用偏移页表

### 创建新映射

### 分配内存帧

## 总结

## 接下来?

