# Create the runtime image
FROM ubuntu:20.04

COPY ./target/release/server /usr/local/bin/wlog-server
COPY ./server/Rocket.toml /root

WORKDIR /root

# Start the helloworld service on container boot
CMD ["/usr/local/bin/wlog-server"]
