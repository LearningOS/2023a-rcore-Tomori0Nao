# lab1 实验报告
## 实现的功能
- 在"/os/src/syscall/mod.rs"中在正式执行系统调用前，记录该系统调用号"syscall_id"，实现记录各系统调用调用次数功能
- 在"/os/src/syscall/process.rs"中
    - 完成"sys_task_info"系统调用，将当前任务的TaskInfo赋值给传入的引用
- 在"/os/src/task/mod.rs"中
    - 为"TASK_MANAGER"中的Task添加TaskInfo成员的初始化
    - 添加"set_syscall_times"函数，在进行系统调用时，为当前的"TaskControlBlock"，记录系统调用次数
    - 添加"get_task_info"函数，获取当前任务的TaskInfo，为添加的函数增加相应接口
- 在"/os/src/task/task.rs"中
    - 添加"TaskInfo"结构体的定义
    - 为"TaskControlBlock"结构体添加"TaskInfo"类型的成员变量

## 问答题
1. 报错信息: 
``` log
[ERROR] [kernel] .bss [0x8027b000, 0x802a4000)
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```
RustSBI版本: "0.3.0-alpha.2"

2. 
    1. 
        - "__alltrps"中通过"mv a0, sp"指令，使得a0中保存TrapContext,而进入"__restore"时，a0的值未发生修改，仍然为TrapContext
        - "__restore"用于内核态和用户态相互转换过程中内核栈/用户栈上下文的恢复 或 函数调用过程中，函数上下文的恢复
    2. 
        - **sstatus**寄存器 SPP 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息，用于判断CPU特权级
        - **sepc**寄存器 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址，用于trap结束后继续执行原程序
        - **sscratch**寄存器 作为中转寄存器，保存内核栈地址，或实现其他寄存器数据交换
    3. 
        - 跳过x2: 用户栈的栈指针保存在 sscratch 中，必须通过csrr指令读到通用寄存器中后才能使用，因此我们先考虑保存其它通用寄存器，腾出空间。
        - 跳过x4: tp(x4) 寄存器 一般不会使用到，无需保存其值
    4.  
        - "csrrw sp, sscratch, sp"是一条原子指令，实现 sp 与 sscratch 寄存器中值的互换，此指令执行后，sp指向用户栈, sscratch 指向内核栈
    5. 
        -  从U态进入S态是通过"__alltrps"中的"csrrw sp, sscratch, sp"指令发生的，实现用户栈到内核栈的转变
## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    - https://werifu.github.io/posts/rcore-camp-2022-lab1/
    - https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/6answer.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。