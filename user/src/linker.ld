OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80400000;

SECTIONS
{
    . = BASE_ADDRESS; # 设置当前代码段的起始地址为 0x80400000
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .; # 在大括号内获取的当前位置.表示bss的结束位置,若在大括号外则表示bss结束后的4字节对齐位置
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}