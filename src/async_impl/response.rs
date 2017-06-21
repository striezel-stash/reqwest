use std::fmt;
use std::marker::PhantomData;

use futures::{Async, Future, Poll, Stream};
use futures::stream::Concat2;
use header::Headers;
use hyper::StatusCode;
use serde::de::DeserializeOwned;
use serde_json;
use url::Url;

use super::{body, Body};


/// A Response to a submitted `Request`.
pub struct Response {
    status: StatusCode,
    headers: Headers,
    url: Url,
    body: Body,
}

impl Response {
    /// Get the final `Url` of this `Response`.
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the `StatusCode` of this `Response`.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the `Headers` of this `Response`.
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` of this `Response`.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Get a mutable reference to the `Body` of this `Response`.
    #[inline]
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// Try to deserialize the response body as JSON using `serde`.
    #[inline]
    pub fn json<T: DeserializeOwned>(&mut self) -> Json<T> {
        Json {
            concat: body::take(self.body_mut()).concat2(),
            _marker: PhantomData,
        }
    }
}


impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Response")
            .field("url", self.url())
            .field("status", &self.status())
            .field("headers", self.headers())
            .finish()
    }
}

pub struct Json<T> {
    concat: Concat2<Body>,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned> Future for Json<T> {
    type Item = T;
    type Error = ::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let bytes = try_ready!(self.concat.poll());
        let t = try_!(serde_json::from_slice(&bytes));
        Ok(Async::Ready(t))
    }
}

impl<T> fmt::Debug for Json<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Json")
            .finish()
    }
}

// pub(crate)

pub fn new(mut res: ::hyper::client::Response, url: Url, _gzip: bool) -> Response {
    use std::mem;

    let status = res.status();
    let headers = mem::replace(res.headers_mut(), Headers::new());
    let body = res.body();
    info!("Response: '{}' for {}", status, url);
    Response {
        status: status,
        headers: headers,
        url: url,
        body: super::body::wrap(body),
    }
}
