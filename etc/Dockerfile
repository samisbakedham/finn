FROM rust:latest

# Prevent prompts
RUN echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections

RUN apt -y update && apt -y upgrade

RUN apt -y install curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN apt -y install build-essential cmake git libgit2-dev clang libncurses5-dev libncursesw5-dev zlib1g-dev pkg-config libssl-dev llvm

WORKDIR /root

RUN git clone https://github.com/finnproject/finn-node.git

WORKDIR /root/finn-node

RUN cargo build --release

RUN cp target/release/finn /usr/bin

CMD ["/usr/bin/finn"]

