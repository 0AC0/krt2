#!/bin/bash

killall qemu-system-riscv64 || echo "Nothing to kill."
qemu-system-riscv64 -vnc localhost:10344 -m 48M      	\
    -serial stdio \
    -trace "*virt*" \
	-device virtio-gpu-device						        \
    -machine virt							                \
	-bios target/riscv64imac-unknown-none-elf/debug/krt2 \
    -gdb tcp::3333 -S

    # -serial mon:telnet::4444,server,nowait \