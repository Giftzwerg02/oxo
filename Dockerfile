FROM rust:1-bullseye as build
WORKDIR /app
COPY . /app
COPY --from=mwader/static-ffmpeg:6.0 /ffmpeg /ffmpeg

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt-get update && \
    apt-get install -y upx libopus-dev cmake
RUN if [ $(uname -m) = "x86_64" ]; then YTDLP_BIN_NAME=yt-dlp_linux; \
    else YTDLP_BIN_NAME=yt-dlp_linux_$(uname -m); fi && \
    wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/$YTDLP_BIN_NAME -O /yt-dlp && \
    chmod +x /yt-dlp
RUN cargo build --release && \
    upx --lzma --best /app/target/release/oxo && \
    upx -1 /ffmpeg && \
    upx -1 /yt-dlp

FROM gcr.io/distroless/cc:nonroot
WORKDIR /app

COPY --from=build /app/target/release/oxo /app/oxo
COPY --from=build /ffmpeg /bin/
COPY --from=build /yt-dlp /bin/

USER nonroot

CMD [ "/app/oxo" ]
