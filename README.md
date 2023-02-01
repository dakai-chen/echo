<h1 align="center">
    echo
</h1>

<p align="center">
    简单易用的异步网络框架
</p>

## 支持

- [x] Multipart
- [x] Server-Sent Events (SSE)
- [x] WebSocket

## 快速开始

新建一个项目：

```bash
> cargo new demo && cd demo
```

将依赖项添加到`Cargo.toml`：

```toml
echo = { git = "https://github.com/dakai-chen/echo.git" }
tokio = { version = "1", features = ["full"] }
```

用以下内容覆盖`src/main.rs`：

```rust
use std::convert::Infallible;

use echo::response::IntoResponse;
use echo::server::Server;
use echo::service::{service_fn, ServiceFn};
use echo::Request;

#[tokio::main]
async fn main() {
    Server::bind("127.0.0.1:3000".parse().unwrap())
        .serve::<ServiceFn<_>>(service_fn(hello))
        .await
        .unwrap();
}

async fn hello(_: Request) -> Result<impl IntoResponse, Infallible> {
    Ok("Hello, World!")
}
```

运行项目：

```bash
> cargo run
```

访问服务：

```bash
> curl http://127.0.0.1:3000/
Hello, World!
```

## 示例

更多示例可以在[这里](./examples/)找到。
