# 开发日志

**OS运行方式**：在os目录下使用`make run`命令可以直接开始运行，基于qemu7.0.0模拟器

## 2024.04.11

- 目录介绍
  ```
    .  
    ├── bootloader----------------------（rustsbi-qemu启动文件）  
    ├── os------------------------------（kernel）  
    ├── README.md  
    └── user----------------------------（用户程序）
  ``` 
- 完成ch2分支推送，目前main分支上是一个简易的批处理操作系统
- 创建个人控制分支“Ben”

## 2024.05.15

- 基本完成ch3内容，实现多道程序以及时间片轮转的操作系统
- 支持浮点运算
- 支持内核中断响应
- 支持统计程序执行时间
  
## 2024.05.14

- 在main函数中设置开始的时钟中断
- 修改`Trap.S`中有关判断中断来源的代码，当前并未实现地址空间，因此需要使用`tp`寄存器保存`spp`字段
  
## 2024.05.21

- 实现操作系统的内存虚拟化，并为上层用户程序提供地址空间
- 添加`sbrk`系统调用
