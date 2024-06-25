FROM rust:1.79 AS build

# Copying the source code
WORKDIR /quantum-node
RUN rustup target add x86_64-unknown-linux-musl
COPY . .

# for allowing cargo to fetch git repos
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# updating and installing
RUN apt-get update && apt-get install -y openssh-client git clang llvm python3 python3-pip python3-venv tmux


# installing go
RUN wget https://go.dev/dl/go1.22.4.linux-amd64.tar.gz
RUN  tar -C /usr/local -xzf go1.22.4.linux-amd64.tar.gz
ENV PATH=$PATH:/usr/local/go/bin
RUN go version

# Adding github configuration
ARG GITHUB_PAT
RUN git config --global url."https://${GITHUB_PAT}:@github.com/".insteadOf "https://github.com/"

# Verify Clang installation and list libraries
RUN ls -l /usr/lib/llvm-14/lib
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

# cloning quantum circuits
WORKDIR /
RUN git clone https://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@github.com/Electron-Labs/quantum-circuits.git
WORKDIR /quantum-circuits
RUN go mod tidy
WORKDIR /quantum-circuits/quantum_circuits_ffi
RUN cargo install rust2go-cli
WORKDIR /quantum-circuits
RUN rust2go-cli --src quantum_circuits_ffi/src/circuit_builder.rs --dst ./gen.go



WORKDIR /quantum-circuits/quantum_circuits_ffi
RUN cargo build --release

WORKDIR /quantum-node
RUN cargo build --release 
RUN ls -la /quantum-node/target/release/quantum_api_server

# quantum_api_server image
FROM rust:1.79 AS quantum_api_server
COPY --from=build /quantum-node/target/release/quantum_api_server .
COPY --from=build /quantum-node/Rocket.toml .
EXPOSE 8000
COPY --from=build /quantum-node/config.yaml /quantum-node/config.yaml
CMD ["./quantum_api_server"]
LABEL service=quantum_api_server

# quantum_worker image
FROM rust:1.79 AS quantum_worker
COPY --from=build /quantum-node/target/release/quantum_worker .
COPY --from=build /quantum-node/config.yaml /quantum-node/config.yaml
CMD ["./quantum_worker"]
LABEL service=quantum_worker
