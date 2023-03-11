FROM rust:1-slim-bullseye as builder
WORKDIR /app
COPY . /app

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV LIBOPUS_STATIC=true

RUN apt-get update && \
    apt-get install -y libopus-dev cmake wget
RUN if [ $(uname -m) = "x86_64" ]; then YTDLP_BIN=yt-dlp_linux; \
    else YTDLP_BIN=yt-dlp_linux_$(uname -m); fi && \
    wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/$YTDLP_BIN -O /yt-dlp && \
    chmod +x /yt-dlp
RUN cargo build --release


FROM alpine:3 AS compressor

COPY --from=mwader/static-ffmpeg:6.0 /ffmpeg /ffmpeg
COPY --from=builder /app/target/release/oxo /oxo
COPY --from=builder /yt-dlp /yt-dlp

RUN apk add upx wget && \
    upx --lzma --best /oxo && \
    upx -1 /yt-dlp && \
    upx -1 /ffmpeg


FROM gcr.io/distroless/cc:nonroot

COPY --from=compressor /oxo /bin/
COPY --from=compressor /ffmpeg /bin/
COPY --from=compressor /yt-dlp /bin/

USER nonroot

CMD [ "oxo" ]
