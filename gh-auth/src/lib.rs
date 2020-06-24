use anyhow::Result;
use url::form_urlencoded::{parse, Serializer};

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

pub async fn start_auth_flow() -> Result<String> {
    let auth_flow = OAuthFlow::new(
        OAUTH_HOST,
        OAUTH_CLIENT_ID,
        OAUTH_CLIENT_SECRET,
        vec!["repo", "read:org", "workflow", "gist"],
        SUCCESS_HTML,
    );
    auth_flow.obtain_access_token().await
}

/// Roughly based on cli/cli the official gh cli implementation in Go!
/// https://github.com/cli/cli/blob/658d548c5e690b4fb4dd6ac06d4b798238b6157f/auth/oauth.go#L29
pub struct OAuthFlow<'a> {
    host_name: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    scopes: Vec<&'a str>,
    success_html: &'a str,
}

impl OAuthFlow<'_> {
    pub fn new<'a>(
        host_name: &'a str,
        client_id: &'a str,
        client_secret: &'a str,
        scopes: Vec<&'a str>,
        success_html: &'a str,
    ) -> OAuthFlow<'a> {
        OAuthFlow {
            host_name,
            client_id,
            client_secret,
            scopes,
            success_html,
        }
    }

    pub async fn obtain_access_token(&self) -> Result<String> {
        let state = utils::generate_random_string(20);
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", self.client_id)
            .append_pair("redirect_uri", &format!("http://{}/callback", addr))
            .append_pair("scope", &self.scopes.join(" "))
            .append_pair("state", &state)
            .finish();

        let url = hyper::Uri::builder()
            .scheme("https")
            .authority(self.host_name)
            .path_and_query(format!("/login/oauth/authorize?{}", query).as_str())
            .build()?
            .to_string();

        if webbrowser::open(&url).is_err() {
            // handle browser not open case
            eprintln!("[Auth] Error opening the brower\n");
            eprintln!("Please open the following URL manually:\n{}\n", url);
        }

        let access_token = {
            use futures::{channel::oneshot, lock::Mutex};
            use hyper::{
                body::to_bytes,
                service::{make_service_fn, service_fn},
                Body, Client, Method, Request, Response, Server, StatusCode, Uri,
            };
            use std::sync::Arc;

            let (shutdown_server_send, shutdown_server_recv) = oneshot::channel::<()>();
            let (auth_code_send, auth_code_recv) = oneshot::channel::<(String, String)>();
            let auth_code_send = Arc::new(Mutex::new(Some(auth_code_send)));

            let original_state = &state;
            let success_html = self.success_html;

            let service = make_service_fn(move |_| {
                let auth_code_send = auth_code_send.clone();
                let original_state = original_state.to_owned();
                let success_html = success_html.to_owned();

                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let auth_code_send = auth_code_send.clone();
                        let original_state = original_state.to_owned();
                        let success_html = success_html.to_owned();

                        async move {
                            match (req.method(), req.uri().path()) {
                                (&Method::GET, "/callback") => {
                                    // Get `state` & `code` from query.
                                    let query = req.uri().query().unwrap_or("");
                                    let parsed = parse(query.as_bytes());
                                    let state_recvd = utils::get_item_from_parse(parsed, "state");
                                    if state_recvd == original_state {
                                        if let Some(auth_code_send) =
                                            auth_code_send.lock().await.take()
                                        {
                                            let code = utils::get_item_from_parse(parsed, "code");
                                            let _ = auth_code_send.send((code, state_recvd));
                                        }
                                        Ok::<_, hyper::Error>(Response::new(Body::from(
                                            success_html,
                                        )))
                                    } else {
                                        Ok::<_, hyper::Error>(Response::new(Body::from(
                                            "Error: state mismatch",
                                        )))
                                    }
                                }
                                _ => {
                                    let mut not_found = Response::default();
                                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                                    Ok::<_, hyper::Error>(not_found)
                                }
                            }
                        }
                    }))
                }
            });

            let server = Server::from_tcp(listener)?.serve(service);
            let graceful = server.with_graceful_shutdown(async {
                shutdown_server_recv.await.ok();
            });

            let capture_code = async {
                let (code, state) = auth_code_recv.await?;
                let _ = shutdown_server_send.send(());
                let token_url = Uri::builder()
                    .scheme("https")
                    .authority(self.host_name)
                    .path_and_query("/login/oauth/access_token")
                    .build()?;
                let client = Client::builder().build::<_, Body>(hyper_rustls::HttpsConnector::new());
                let form_encoded = Serializer::new(String::new())
                    .append_pair("client_id", self.client_id)
                    .append_pair("client_secret", self.client_secret)
                    .append_pair("code", &code)
                    .append_pair("state", &state)
                    .finish();
                let req = Request::builder()
                    .method(Method::POST)
                    .uri(token_url)
                    .body(Body::from(form_encoded))?;
                let resp = client.request(req).await?;
                let resp = to_bytes(resp.into_body()).await?;
                let parsed = parse(&resp);
                let access_token = utils::get_item_from_parse(parsed, "access_token");
                Ok::<_, anyhow::Error>(access_token)
            };

            println!("[Auth] Listening on http://{}", addr);
            let (graceful, access_token) = futures::join!(graceful, capture_code);

            if let Err(e) = graceful {
                eprintln!("[Auth] Server error: {}", e);
            }

            access_token?
        };

        Ok(access_token)
    }
}

mod utils {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    use url::form_urlencoded::Parse;

    pub(crate) fn generate_random_string(length: usize) -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(length)
            .collect()
    }

    pub(crate) fn get_item_from_parse(iter: Parse, key: &str) -> String {
        iter.clone()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.to_string())
            .unwrap_or_default()
    }
}
