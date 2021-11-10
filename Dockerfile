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

ENV CARGO_HOME /cargo
ENV RUSTUP_HOME /rustup
RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o run.sh \
	&& chmod a+x run.sh \
	&& ./run.sh -y && rm run.sh
ENV PATH $PATH:/cargo/bin
RUN chmod -R 777 /cargo /rustup
COPY ci/cargo-config /cargo/config

RUN mkdir -p $HOME/megflow-runspace
WORKDIR $HOME/megflow-runspace
COPY . $HOME/megflow-runspace/

RUN cargo build \
    && cd flow-python \
    && python3 setup.py install --user \
    && cd examples \
    && cargo run --example megflow_run -- -p logical_test

RUN cargo build --example megflow_run --release \
    && ln -s target/release/examples/megflow_run
