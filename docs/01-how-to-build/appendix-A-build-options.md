# Build options

| cargo features | function |
| --------- | ----------- |
| open-camera                | open camera via v4l2 on VideoServer          |
| no-default-features    | build without rweb/ffmpeg/decoder           |

| environment | function |
| --------- | ----------- |
| CARGO_FEATURE_PREBUILD | use prebuild ffmpeg      |
| CARGO_FEATURE_STATIC        | build static ffmpeg lib   |
| FFMPEG_DIR        | specify prebuild ffmpeg dir   |
