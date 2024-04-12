# Use Ubuntu jammy as the base image
FROM ubuntu:jammy

# Install necessary packages
RUN apt-get update && apt-get install -y \
    curl \
    gnupg \
    lsb-release \
    software-properties-common \
    build-essential \
    m4 \
    zlib1g-dev \
    libzstd-dev

# Prepare to add LLVM repository
RUN curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /usr/share/keyrings/llvm-archive-keyring.gpg

# Add the LLVM repository using the modern signed-by method
RUN echo "deb [signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] http://apt.llvm.org/jammy llvm-toolchain-jammy-17 main" > /etc/apt/sources.list.d/llvm.list

# Install LLVM 17
RUN apt-get update && apt-get install -y llvm-17 llvm-17-dev clang-17 libpolly-17-dev
ENV CC=clang-17
ENV CXX=clang++-17

# Install Rust nightly
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Configure cargo to use clang-17 as the linker
RUN mkdir -p ~/.cargo \
    && echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang-17"' > ~/.cargo/config.toml

# Clean up
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

# Build ppl
WORKDIR /usr/src/ppl
COPY . .
RUN cargo build

CMD ["cargo", "test", "--verbose"]
