FROM clux/muslrust:stable as build
# Install build deps package
RUN cargo install cargo-bdeps
# Set workdir to root
WORKDIR /
# Create sample project
RUN USER=root cargo new --bin live-track
# Make it the workdir
WORKDIR /live-track
# Copy over toml files
COPY Cargo.toml Cargo.lock ./
# Build the dependencies
RUN cargo-bdeps --release
# Build will be cached up to here unless Cargo.toml is updated

# Copy over all project files
COPY src src
# Build the whole thing
RUN cargo build --release --bin live-track
# Copy over the static content
COPY static static
# Copy over to other container
RUN mkdir -p move
RUN cp -r static move/
RUN cp /live-track/target/x86_64-unknown-linux-musl/release/live-track move/
RUN strip move/live-track

FROM gcr.io/distroless/static
COPY --from=build /live-track/move /
EXPOSE 8080
CMD ["/live-track"]
