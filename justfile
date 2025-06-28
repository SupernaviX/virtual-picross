build:
    cargo build --release && \
    cargo objcopy --release -- -S -O binary virtual-picross.vb
assembly:
    cargo rustc --release -- --emit asm --emit llvm-ir
    cargo objdump --release -- --disassemble >virtual-picross.s