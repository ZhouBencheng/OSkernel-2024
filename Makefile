TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin

build:
	cargo build --release
	rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

run:
	qemu-system-riscv64 \
		-machine virt\
	 	-nographic\
		-bios ../bootloader/rustsbi-qemu.bin\
		-device loader,file=$(KERNEL_BIN),addr=0x80200000

clean:
	cargo clean
