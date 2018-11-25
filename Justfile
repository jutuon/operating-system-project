# test

run: build run-cmd
run-release: build-release run-cmd

run-cmd:
    qemu-system-i386 -m 32M -boot order=d -cdrom build/grub.iso -cpu n270 -d int,cpu_reset -no-reboot

build: build-debug-binary create-grub-iso
build-release: build-release-binary create-grub-iso-release


@build-debug-binary:
    cargo xrustc --target i686-unknown-none.json -- -C link-args="--script linker-script.ld"
@build-release-binary:
    cargo xrustc --release --target i686-unknown-none.json -- -C link-args="--script linker-script.ld"

@create-grub-iso:
    mkdir -p build/iso/boot/grub 2> /dev/null | true
    cp grub.cfg build/iso/boot/grub
    cp target/i686-unknown-none/debug/operating-system-project build/iso/boot/kernel.bin
    grub-mkrescue -o build/grub.iso build/iso
@create-grub-iso-release:
    mkdir -p build/iso/boot/grub 2> /dev/null | true
    cp grub.cfg build/iso/boot/grub
    cp target/i686-unknown-none/release/operating-system-project build/iso/boot/kernel.bin
    grub-mkrescue -o build/grub.iso build/iso


clean:
	rm -fr build
