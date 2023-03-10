# Nightly is required until `-Z sparse-registry` is stabilized in Rust 1.68
# https://github.com/rust-lang/cargo/issues/9069#issuecomment-1408773982
FROM rustlang/rust:nightly-slim as build
WORKDIR /app
COPY . /app
COPY --from=mwader/static-ffmpeg:5.1.2 /ffmpeg /ffmpeg
COPY --from=jauderho/yt-dlp:latest /usr/local/bin/yt-dlp /yt-dlp

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt-get update && \
    apt-get install -y upx libopus-dev cmake
RUN cargo build --release && \
    upx --lzma --best /app/target/release/oxo && \
    upx -1 /ffmpeg
    # Note: yt-dlp appears to be not compressable

FROM gcr.io/distroless/cc:nonroot
WORKDIR /app

COPY --from=build /app/target/release/oxo /app/oxo
COPY --from=build /ffmpeg /bin/
COPY --from=build /yt-dlp /bin/

USER nonroot

CMD [ "/app/oxo" ]
