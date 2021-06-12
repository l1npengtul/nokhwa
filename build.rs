fn main() {
    if cfg!(feature = "input-ffmpeg") && option_env!("FFMPEG_PKG_CONFIG_PATH").is_none() {
        std::env::set_var("FFMPEG_PKG_CONFIG_PATH", "");
    }
}
