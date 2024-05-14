.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps: # 保存上下文
    csrrw sp, sscratch, sp
    # 交换 sp->kernel stack, sscratch->user stack
    # 在内核栈上分配一帧保存TrapContext
    addi sp, sp, -34*8
    # 以下保存通用寄存器
    sd x1, 1*8(sp)
    # 暂时越过 sp(x2),稍后保存
    sd x3, 3*8(sp)
    # 忽略 tp(x4), 用户程序用不到这个寄存器
    # 保存 x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 此时我们可以自由使用 t0/t1/t2，因为他们已经被保存在内核栈上
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 读取用户栈地址
    csrr t2, sscratch
    # 将用户栈地址保存在内核栈上
    sd t2, 2*8(sp)
    # 获取内核栈栈顶指针（指向内核栈刚保存的TrapContext）
    mv a0, sp
    call trap_handler

__restore: # 恢复上下文
    # case1: app运行环境初始化，在lazy_static!初始化TaskManager过程中，对每个Task调用goto_restore设置任务上下文
    # case2: trap处理后返回用户态
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret
