build:
    cargo build --release && \
    cargo objcopy --release -- -S -O binary virtual-picross.vb