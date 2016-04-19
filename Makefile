LD=ld
RUSTC=rustc
NASM=nasm
QEMU=qemu-system-i386

all: harddrive.img

.SUFFIXES: .o .rs .asm

.PHONY: clean run

.rs.o:
	$(RUSTC) -O -A dead-code -C relocation-model=dynamic-no-pic -C no-stack-check -Z no-landing-pads --target i386-unknown-redox.json --crate-type lib -L . -o $@ --emit obj $<

.asm.o:
	$(NASM) -f elf32 -o $@ $<

harddrive.img: loader.asm kernel.bin
	$(NASM) -o $@ -l loader.lst -f bin $<

kernel.bin: linker.ld kernel.o
	$(LD) -m elf_i386 -o $@ -T $^

run: harddrive.img
	$(QEMU) -serial mon:stdio -sdl -hda $<

clean:
	rm -f *.bin *.o *.img *.lst *.map
