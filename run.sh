#!/bin/sh

daemonize -e stderr -o stdout -c . $(which qemu-system-riscv64) -m 48M      	\
    -trace "*virt*"\
    -trace "*queue*"\
	-device virtio-gpu-device						        \
    -machine virt							                \
	-bios target/riscv64imac-unknown-none-elf/debug/krt2 \
    -gdb tcp::3333 -S
