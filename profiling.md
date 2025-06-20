> This file is currently only availible in Chinese(Simplified).
# 使用Tracy检测rust程序性能指北

## 1. 下载Tracy

不同平台的Tracy下载方式各不相同，  
可以前往 [Tracy Releases](https://github.com/wolfpld/tracy/releases) 下载Windows的预编译包。下载后解压，得到 `Tracy.exe` 可执行文件。其他平台可以参见[Tracy Documentation](https://github.com/wolfpld/tracy/releases/latest/download/tracy.pdf).

`tracing-tracy` 和 Tracy 的兼容性表可以在[crates.io](https://crates.io/crates/tracing-tracy)找到。

## 2. 集成tracing-tracy到Rust项目

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
tracing = "0.1"
tracing-tracy = "0.11"
```

在代码中初始化 TracyLayer 并添加性能采集点：

```rust
// filepath: src/main.rs
use tracing::{info, span, Level};
use tracing_tracy::TracyLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

fn main() {
    let tracy_layer = TracyLayer::new();
    let subscriber = Registry::default().with(tracy_layer);
    tracing::subscriber::set_global_default(subscriber).expect("设置全局 subscriber 失败");

    let main_span = span!(Level::INFO, "main");
    let _enter = main_span.enter();

    info!("程序启动");
    // 需要分析性能的代码
}
```

## 3. 使用tracy-capture生成`.tracy`文件

如果你希望将性能数据保存为文件（如 `xxx.tracy`），便于后续分析，可以使用 [tracy-capture](https://github.com/wolfpld/tracy/tree/master/capture) 工具, 它是tracy的一部分。不同平台的文件名可能不同，如 `capture.exe` (Windows)、`tracy` (MacOS)、`capture-release` 或者 `tracy-capture` (Linux)。

### 3.1 采集性能数据

使用 `tracy-capture` 采集数据：

```sh
tracy-capture --output my_capture.tracy
```
它会监听一个本地地址，默认为`127.0.0.1:8086`, 我们的程序会向这个地址发送数据包。
然后启动被监听的程序，这样会在程序运行结束后生成 `my_capture.tracy` 文件。

### 3.3 使用Tracy客户端打开`.tracy`文件

1. 启动 Tracy 客户端。
2. 选择“File” -> “Open Capture...”。
3. 打开刚刚生成的 `my_capture.tracy` 文件，即可离线分析性能数据。

或者直接在命令行输入`tracy /path/to/tracyfile.tracy`。

## 4. 常见问题

- **Tracy 客户端未显示数据？**  
  检查防火墙设置，确保端口未被阻塞。确认 `tracing-tracy` 已正确初始化。
- **性能数据不全？**  
  检查是否在关键代码段添加了 span。

## 5. 参考资料

- [tracing-tracy crate 文档](https://docs.rs/tracing-tracy)
- [Tracy 官方文档](https://github.com/wolfpld/tracy)
- [Bevy 性能检测指南](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md)