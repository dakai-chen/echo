use echo_core::http::Extensions;
use matchit::Params;

use super::router::PRIVATE_TAIL_PARAM;

/// 路由器提取的路径参数。
#[derive(Debug, Clone)]
pub struct PathParams(Vec<(String, String)>);

impl PathParams {
    pub fn get_ref(&self) -> &Vec<(String, String)> {
        &self.0
    }

    pub fn into_inner(self) -> Vec<(String, String)> {
        self.0
    }
}

pub(crate) fn prase_path_params(params: Params) -> (Vec<(String, String)>, Option<String>) {
    params.iter().fold(
        (Vec::with_capacity(params.len()), None),
        |(mut params, mut tail), (k, v)| {
            if k == PRIVATE_TAIL_PARAM {
                tail = Some(format!("/{}", if v.starts_with('/') { &v[1..] } else { v }));
            } else {
                params.push((k.to_owned(), v.to_owned()));
            }
            (params, tail)
        },
    )
}

pub(crate) fn insert_path_params(extensions: &mut Extensions, params: Vec<(String, String)>) {
    let path_params = if let Some(path_params) = extensions.get_mut::<PathParams>() {
        path_params
    } else {
        extensions.insert(PathParams(vec![]));
        extensions.get_mut().unwrap()
    };
    path_params.0.extend(params);
}
