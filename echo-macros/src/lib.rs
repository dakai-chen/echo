mod route;

use proc_macro::TokenStream;

/// 将异步函数包装为路由，并允许设置多个HTTP访问方法。
///
/// # 例子
///
/// ```
/// # use echo::response::IntoResponse;
/// # use echo::{BoxError, Request};
/// #[echo::route("/test", method = "GET", method = "POST")]
/// async fn example(_: Request) -> Result<impl IntoResponse, BoxError> {
///     Ok("")
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    route::route(args, input)
}
