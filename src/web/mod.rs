use bytes::{BufMut, BytesMut};
use once_cell::sync::Lazy;
use salvo::{
    http::{header, response::Body, StatusCode},
    Response,
};
use std::io::{self, Write};

mod admin;
mod visitor;

pub use admin::Admin;
pub use visitor::ArticleWeb;

static TERA: Lazy<tera::Tera> = Lazy::new(|| tera::Tera::new("views/**/*").unwrap());

pub(crate) struct Cache(BytesMut);

impl Cache {
    pub fn new() -> Self {
        Cache(BytesMut::with_capacity(1024))
    }

    pub fn with_capacity(size: usize) -> Self {
        Cache(BytesMut::with_capacity(size))
    }

    pub fn into_inner(self) -> BytesMut {
        self.0
    }
}

impl Write for Cache {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.put(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn render(res: &mut Response, path: &str, ctx: &tera::Context) {
    let mut body = Cache::new();
    TERA.render_to(path, &ctx, &mut body).unwrap();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    res.set_status_code(StatusCode::OK);
    res.set_body(Some(Body::Bytes(body.into_inner())))
}
