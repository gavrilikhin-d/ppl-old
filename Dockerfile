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
RUN echo "deb [signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] http://apt.llvm.org/jammy llvm-toolchain-jammy-18 main" > /etc/apt/sources.list.d/llvm.list

# Install LLVM 18
RUN apt-get update && apt-get install -y llvm-18 llvm-18-dev clang-18 libpolly-18-dev
ENV CC=clang-18
ENV CXX=clang++-18

# Install Rust nightly
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Configure cargo to use clang-18 as the linker
RUN mkdir -p ~/.cargo \
    && echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang-18"' > ~/.cargo/config.toml

# Clean up
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

# Build ppl
WORKDIR /usr/src/ppl
COPY . .
RUN cargo build

CMD ["cargo", "test", "--verbose"]
