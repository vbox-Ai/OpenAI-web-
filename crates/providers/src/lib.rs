//! Agent2API 网页提供商插件（库形态）。
//!
//! ── 两种运行形态 ────────────────────────────────────────────
//! 1. **独立进程**（`main.rs`，`bin` feature）：网关 spawn 它，走 stdin/stdout。
//! 2. **进程内嵌入**（本 lib，`staticlib`）：iOS 沙箱禁止 spawn，把实现链进
//!    主二进制，网关通过内存管道直接调 `serve_inprocess`。
//!
//! ── 与 Agent2Providers 的差异 ───────────────────────────────
//! 本包**没有** `paths` 模块（不需要 data_dir 落盘：网页提供商不缓存机器标识 /
//! 组件文件），所以 `on_init` 为 `None`。这也避免了进程内多包共用
//! `paths::DATA_DIR` 的初始化冲突。

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod content_block;
#[cfg(windows)]
mod crypto;
mod errors;
mod headers;
mod http;
mod json;
mod logging;
mod onboarding;
mod providers;
mod quota;
mod store;

use tokio::io::{AsyncRead, AsyncWrite};

pub use providers::registry;

/// 独立进程形态的入口（`main.rs` 调用）。
pub fn run_standalone() {
    agent2api_provider_sdk::serve_blocking(providers::registry());
}

/// 进程内嵌入形态的入口：在当前进程里跑 `serve` 主循环。
///
/// 参数是一对已就绪的 tokio 句柄（读端 + 写端）。网关用
/// `tokio::io::duplex` 建内存管道，一端给本函数、另一端接进自己的读循环。
pub async fn serve_inprocess(
    reader: Box<dyn AsyncRead + Unpin + Send + Sync>,
    writer: Box<dyn AsyncWrite + Unpin + Send + Sync>,
) -> std::io::Result<()> {
    let options = agent2api_provider_sdk::ServeOptions {
        on_init: None,
        reader: Some(reader),
        writer: Some(writer),
        in_process: true,
    };
    agent2api_provider_sdk::serve_with(providers::registry(), options).await
}
