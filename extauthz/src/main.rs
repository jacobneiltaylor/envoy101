#![cfg_attr(not(unix), allow(unused_imports))]

use std::collections::HashMap;
use std::env;
use std::path::Path;
use envoy_types::pb::google::protobuf::value::Kind;
use envoy_types::pb::google::protobuf::{Struct, Value};
use tonic::{transport::Server, Request, Response, Status};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;
#[cfg(unix)]
use tonic::transport::server::UdsConnectInfo;
use envoy_types::ext_authz::v3::pb::{
    Authorization, AuthorizationServer, CheckRequest, CheckResponse, HeaderAppendAction, HttpStatusCode
};
use envoy_types::ext_authz::v3::{
    CheckRequestExt, CheckResponseExt, DeniedHttpResponseBuilder, OkHttpResponseBuilder,
};

use phf::phf_map;

use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

type Metadata = HashMap<String, Value>;

struct BasicAuthorizationServer {
    realm: String,
}

static USER_DB: phf::Map<&'static str, &'static str> = phf_map! {
    "user" => "user",
    "admin" => "admin",
};

fn make_string_value(val: String) -> Value {
    Value {
        kind: Some(Kind::StringValue(val))
    }
}

fn make_bool_value(val: bool) -> Value {
    Value {
        kind: Some(Kind::BoolValue(val))
    }
}

fn wrap_metadata(metadata: Metadata) -> Option<Struct> {
    Some(Struct { fields: metadata })
}

fn decode_base64(data: &str) -> Option<String> {
    match BASE64.decode(data.as_bytes()) {
        Ok(bytes) => {
            match String::from_utf8(bytes) {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

fn decode_basic_token(token: &str) -> Option<(String, String)> {
    match decode_base64(token) {
        Some(data) => {
            let elements: Vec<&str> = data.split(":").collect();

            if elements.len() == 2 {
                return Some((elements[0].to_string(), elements[1].to_string()))
            }
            
            None
        },
        None => None,
    }
}

fn authenticate_client(header_value: &String) -> Result<String, String> {
    let split: Vec<&str> = header_value.split_whitespace().collect();

    if split.len() == 2 {
        let method = split[0];
        let token = split[1];

        let creds = decode_basic_token(token);

        if method == "Basic" && creds.is_some() {
            let (username, password) = creds.unwrap();
            let lookup = USER_DB.get(&username);

            if lookup.is_some() && lookup.unwrap().to_string() == password {
                return Ok(username)
            }
        }
    }

    Err("authorization failed".to_string())
}

fn build_metadata(client_id: String, authenticated: bool) -> Metadata {
    HashMap::from([
        ("identity".to_string(), make_string_value(client_id)),
        ("authenticated".to_string(), make_bool_value(authenticated)),
    ])
}

impl BasicAuthorizationServer {
    fn unauthenticated_response(&self, code: HttpStatusCode, status: Status) -> Response<CheckResponse> {
        let metadata = build_metadata("anonymous".to_string(), false);
        let mut http_response = DeniedHttpResponseBuilder::new();
        http_response
            .set_http_status(code)
            .set_body(status.message());

        if code == HttpStatusCode::Unauthorized {
            http_response.add_header(
                "www-authenticate", 
                format!("Basic realm=\"{}\"", self.realm),
                Some(HeaderAppendAction::OverwriteIfExistsOrAdd), 
                true
            );
        }

        let mut response = CheckResponse::with_status(status);
        response.set_http_response(http_response);
        response.set_dynamic_metadata(wrap_metadata(metadata));

        Response::new(response)
    }

    fn authenticated_response(&self, client_id: String) -> Response<CheckResponse> {
        let metadata = build_metadata(client_id.clone(), true);
        let mut http_response = OkHttpResponseBuilder::new();

        http_response.add_header(
            "x-extauthz-username", 
            client_id, 
            Some(HeaderAppendAction::OverwriteIfExistsOrAdd), 
            true
        );

        let mut ok_response = CheckResponse::with_status(Status::ok("request is valid"));

        ok_response.set_http_response(http_response);
        ok_response.set_dynamic_metadata(wrap_metadata(metadata));

        Response::new(ok_response)
    }
}

impl Default for BasicAuthorizationServer {
    fn default() -> Self {
        BasicAuthorizationServer {
            realm: "envoy".to_string(),
        }
    }
}

#[allow(unused)]
#[tonic::async_trait]
impl Authorization for BasicAuthorizationServer {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        #[cfg(unix)]
        {
            let conn_info = request.extensions().get::<UdsConnectInfo>().unwrap();
            println!("Got a request {:?} with info {:?}", request, conn_info);
        }

        let request = request.into_inner();

        let client_headers = request
            .get_client_headers()
            .ok_or_else(|| Status::invalid_argument("client headers not populated by envoy"))?;

        let client_id = if let Some(authorization) = client_headers.get("authorization") {
            match authenticate_client(authorization) {
                Ok(username) => username,
                Err(msg) => {
                    return Ok(self.unauthenticated_response(
                        HttpStatusCode::Forbidden,
                        Status::unauthenticated(msg),
                    ));
                }
            }
        } else {
            return Ok(self.unauthenticated_response(
                HttpStatusCode::Unauthorized,
                Status::unauthenticated("authorization header not available"),
            ));
        };

        Ok(self.authenticated_response(client_id))
    }
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = env::var("EXTAUTHZ_UDS_PATH").unwrap_or("/var/run/extauthz.sock".into());
    let fnc = filename.clone();
    let path = Path::new(&fnc);

    if path.exists() {
        std::fs::remove_file(path)?;
    }
    
    std::fs::create_dir_all(path.parent().unwrap())?;

    let server = BasicAuthorizationServer::default();

    let uds = UnixListener::bind(filename)?;
    std::fs::set_permissions(
        path, 
        std::os::unix::fs::PermissionsExt::from_mode(0o666),
    )?;

    let uds_stream = UnixListenerStream::new(uds);

    Server::builder()
        .add_service(AuthorizationServer::new(server))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}

#[cfg(not(unix))]
fn main() {
    panic!("This is not a UNIX system. I don't know this.");
}