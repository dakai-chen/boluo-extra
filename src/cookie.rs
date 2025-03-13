//! Cookie 相关的类型和特征。

use std::convert::Infallible;

use boluo_core::extract::FromRequest;
use boluo_core::http::HeaderMap;
use boluo_core::http::header::{COOKIE, SET_COOKIE};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponseParts, ResponseParts};
use either::Either;

pub use cookie::{Cookie, Expiration, SameSite};

/// 从请求中获取 [`Cookie`] 并管理的提取器。
#[derive(Default, Debug, Clone)]
pub struct CookieJar {
    jar: cookie::CookieJar,
}

fn cookies_from_request(
    headers: &HeaderMap,
) -> impl Iterator<Item = Result<Cookie<'static>, CookieParseError>> + '_ {
    headers
        .get_all(COOKIE)
        .iter()
        .flat_map(move |value| match value.to_str() {
            Ok(v) => Either::Left(
                Cookie::split_parse_encoded(v.to_owned())
                    .map(|cookie| cookie.map_err(|e| CookieParseError(e.to_string()))),
            ),
            Err(e) => Either::Right(std::iter::once(Err(CookieParseError(e.to_string())))),
        })
}

impl CookieJar {
    /// 创建一个新的 [`CookieJar`] 实例。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建新的构建器以构建 [`CookieJar`]。
    pub fn builder() -> CookieJarBuilder {
        CookieJar::new().into_builder()
    }

    /// 将当前的 [`CookieJar`] 转换为 [`CookieJarBuilder`]，以便进一步构建。
    pub fn into_builder(self) -> CookieJarBuilder {
        CookieJarBuilder::new(self)
    }

    /// 从请求头中提取 [`Cookie`] 并构建 [`CookieJar`] 实例。
    pub fn from_headers(headers: &HeaderMap) -> Result<Self, CookieParseError> {
        let mut jar = cookie::CookieJar::new();
        for cookie in cookies_from_request(headers) {
            jar.add_original(cookie?);
        }
        Ok(Self { jar })
    }

    /// 根据名称获取指定的 [`Cookie`]。
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.jar.get(name)
    }

    /// 向 [`CookieJar`] 中添加一个新的 [`Cookie`]。
    pub fn add<C: Into<Cookie<'static>>>(&mut self, cookie: C) {
        self.jar.add(cookie);
    }

    /// 从 [`CookieJar`] 中移除指定的 [`Cookie`]。
    pub fn remove<C: Into<Cookie<'static>>>(&mut self, cookie: C) {
        self.jar.remove(cookie);
    }

    /// 返回一个迭代器，用于遍历 [`CookieJar`] 中的所有 [`Cookie`]。
    pub fn iter(&self) -> impl Iterator<Item = &'_ Cookie<'static>> {
        self.jar.iter()
    }
}

impl FromRequest for CookieJar {
    type Error = CookieParseError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        CookieJar::from_headers(req.headers())
    }
}

impl IntoResponseParts for CookieJar {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        for cookie in self.jar.delta() {
            if let Ok(value) = cookie.encoded().to_string().parse() {
                parts.headers.append(SET_COOKIE, value);
            }
        }
        Ok(parts)
    }
}

/// [`CookieJar`] 的构建器。
#[derive(Default, Debug, Clone)]
pub struct CookieJarBuilder {
    jar: CookieJar,
}

impl CookieJarBuilder {
    /// 使用给定的 [`CookieJar`] 创建新的 [`CookieJarBuilder`] 实例。
    pub fn new(jar: CookieJar) -> Self {
        Self { jar }
    }

    /// 向 [`CookieJar`] 中添加一个新的 [`Cookie`]。
    pub fn add<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.add(cookie);
        self
    }

    /// 从 [`CookieJar`] 中移除指定的 [`Cookie`]。
    pub fn remove<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.remove(cookie);
        self
    }

    /// 消耗构建器，返回构建的 [`CookieJar`] 实例。
    pub fn build(self) -> CookieJar {
        self.jar
    }
}

/// Cookie 解析错误。
#[derive(Debug)]
pub struct CookieParseError(String);

impl std::fmt::Display for CookieParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse request header `cookie` ({})", self.0)
    }
}

impl std::error::Error for CookieParseError {}
