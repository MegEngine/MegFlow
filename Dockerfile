FROM ubuntu:18.04

RUN apt update \
    && apt install -y curl \
    && apt install -y ffmpeg \
    && apt install -y yasm \
    && apt install -y clang \
    && apt install -y redis-server \
    && apt install -y python3 \
    && apt install -y python3-pip \
    && apt install -y git \
    && apt install -y build-essential

RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o run.sh \
    && chmod a+x run.sh \
    && ./run.sh -y \
    && export PATH=$HOME/.cargo/bin:${PATH} \
    && cargo --version

RUN mkdir -p $HOME/megflow-runspace
WORKDIR $HOME/megflow-runspace
COPY . $HOME/megflow-runspace/

RUN PATH=$HOME/.cargo/bin:${PATH} \
    && cargo build \
    && cd flow-python \
    && python3 setup.py install --user \
    && cd examples \
    && cargo run --example run_with_plugins -- -p logical_test
