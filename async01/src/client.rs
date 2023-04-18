use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::{Error, IntoUrl, Method, Request, RequestBuilder, Response};

pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let inner = reqwest::Client::builder()
            .user_agent("horo bot/1.0")
            .build()?;

        Ok(Self { inner })
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.inner.request(method, url)
    }

    pub async fn execute(&self, req: Request) -> Result<Response, Error> {
        let exec = || async {
            self.inner
                .execute(req.try_clone().unwrap())
                .await
                .and_then(|r| r.error_for_status())
                .map_err(backoff::Error::transient)
        };

        retry(ExponentialBackoff::default(), exec).await
    }
}
