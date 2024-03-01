# Build stage
FROM rust:1-slim-buster AS build
RUN cargo new --bin app
WORKDIR /app
COPY models /app/models
COPY Cargo.toml /app/
COPY Cargo.lock /app/
# First build to cache dependencies
RUN cargo build --release
COPY src /app/src
# Touch main.rs to ensure we recompile with the actual source code
RUN touch src/main.rs
# Final build with the actual source code
RUN cargo build --release

# Final stage
FROM debian:buster-slim
# Copy the compiled binary from the build stage
COPY --from=build /app/target/release/whatisit /app/whatisit
# Copy the models directory from the build stage
COPY --from=build /app/models /app/models


# Set the working directory to /app
WORKDIR /app

EXPOSE 3000
# Set the command to run your application
CMD ["./whatisit"]