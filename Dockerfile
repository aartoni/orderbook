FROM rust:alpine as builder
WORKDIR /usr/src/orderbook
COPY . .
RUN sed -i "s/files/\/files/" src/main.rs
# Uncomment the following line to enable trades
# RUN sed -i "s/\/\/X//" src/order_book.rs
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/orderbook /usr/local/bin/orderbook
COPY --from=builder /usr/src/orderbook/files/input_file.csv /files/input_file.csv
# Uncomment the following line to execute the extras (remember to enable trades first)
# COPY --from=builder /usr/src/orderbook/files/input_file_extra.csv /files/input_file.csv
CMD ["orderbook"]
