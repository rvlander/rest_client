#![feature(core)]
#![feature(rustc_private)]

extern crate hyper;
extern crate url;
extern crate core;

use hyper::Url;
use hyper::client::Request;
use hyper::method::Method::{Get, Delete, Put, Post, Patch};

use std::fmt::Formatter;
use std::fmt::Display;
use std::io::Error;
use url::ParseError;
use hyper::HttpError;
use hyper::header::ContentLength;
use hyper::header::ContentType;

use hyper::mime::Mime;

pub struct RestClient;

use core::num::ToPrimitive;
use std::io::Write;
use std::io::Read;

impl RestClient {
    // TODO: add cookies

    pub fn get(url_str:&str) -> Result<Response, RestError> {
        RestClient::new(Get, url_str, None, None, None)
    }

    pub fn get_with_params(url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        RestClient::new(Get, url_str, Some(params), None, None)
    }    
 
    pub fn post_with_params(url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        RestClient::pstar_with_params( Post, url_str, params )
    }

    pub fn post(url_str:&str, body:&str, content_type:&str) -> Result<Response, RestError> {
        RestClient::pstar( Post, url_str, body, content_type )
    }

    pub fn patch_with_params(url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        RestClient::pstar_with_params( Patch, url_str, params )
    }

    pub fn patch(url_str:&str, body:&str, content_type:&str) -> Result<Response, RestError> {
        RestClient::pstar( Patch, url_str, body, content_type )
    }
    
    pub fn put_with_params(url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        RestClient::pstar_with_params( Put, url_str, params )
    }

    pub fn put(url_str:&str, body:&str, content_type:&str) -> Result<Response, RestError> {
        RestClient::pstar( Put, url_str, body, content_type )
    }

    pub fn delete(url_str:&str) -> Result<Response, RestError> {
        RestClient::new(Delete, url_str, None, None, None)
    }

    pub fn delete_with_params(url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        RestClient::new(Delete, url_str, Some(params), None, None)
    }    

    fn pstar_with_params( method:hyper::method::Method, url_str:&str, params:&[(&str, &str)]) -> Result<Response, RestError> {
        let post_body = url::form_urlencoded::serialize(params.to_vec().into_iter());

        RestClient::pstar( method, url_str, post_body.as_slice(), "application/x-www-form-urlencoded" )
    }

    fn pstar(method:hyper::method::Method, url_str:&str, body:&str, content_type:&str) -> Result<Response, RestError> {
        RestClient::new( method, url_str, None, Some(body), Some(content_type) )
    }

    pub fn new(method:hyper::method::Method, url_str:&str, url_params:Option<&[(&str, &str)]>, body:Option<&str>, content_type:Option<&str>) -> Result<Response, RestError> {
        let mut url = match Url::parse(url_str) {
            Ok(url) => url,
            Err(err) => return Err(RestError::UrlParseError(err))
        };

        match url_params {
            Some(params) => {
                // TODO: write article talking about iter() vs into_iter()
                url.set_query_from_pairs(params.to_vec().into_iter());
            },
            None => ()
        };

        let mut req = match Request::new(method, url) {
            Ok(req) => req,
            Err(err) => return Err(RestError::HttpRequestError(err))
        };

        match body {
            Some(body) =>
                req.headers_mut().set(ContentLength(body.len() as u64)),
            None => 
                // needed so that hyper doesn't try to send Transfer-Encoding:
                // Chunked, which causes some servers (e.g. www.reddit.co) to
                // hang. is this a bug in the hyper client? why would it send
                // T-E: Ch as a header in a GET request?
                req.headers_mut().set(ContentLength(0))
        };

        match content_type {
            Some (a) => req.headers_mut().set(ContentType(a.parse().unwrap())),
            None => (),
        };

        let mut req_started = match req.start() {
            Ok(req) => req,
            Err(err) => return Err(RestError::HttpRequestError(err))
        };

        let mut void = ();
        match body {
            Some(body) =>
                match req_started.write(body.as_bytes()) {
                    Ok(void) => (),
                    Err(err) => return Err(RestError::HttpIoError(err))
                },
            None => ()
        };

        let mut resp = match req_started.send() {
            Ok(resp) => resp,
            Err(err) => return Err(RestError::HttpRequestError(err))
        };


        let mut body = String::new();
        match resp.read_to_string(&mut body) {
            Ok(body) => body,
            Err(err) => return Err(RestError::HttpIoError(err))
        };

        let rest_response = Response {
            code: resp.status.to_i32().unwrap(),
            status: resp.status,
            headers: resp.headers,
            body: body,
        };

        return Ok(rest_response);
    }
}

pub struct Response {
    pub code: i32,
    pub status: hyper::status::StatusCode,
    pub headers: hyper::header::Headers,
    pub body: String,
}

impl Display for Response {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        self.body.fmt(fmt)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RestError {
    UrlParseError(url::ParseError),
    HttpRequestError(HttpError),
    HttpIoError(Error)
}

#[cfg(test)]
mod test {
    extern crate serialize;
    use super::RestClient;
    use self::serialize::json;

    #[test]
    fn test_get() {
        let response = RestClient::get("http://www.reddit.com/hot.json?limit=1").unwrap();
        let response_json = json::from_str(response.body.as_slice()).unwrap();
        assert!(response_json.find(&"data".to_string()).unwrap().find(&"children".to_string()).unwrap().as_array().unwrap().len() == 1);
    }   
    
    #[test]
    fn test_get_with_params() {
        let response = RestClient::get_with_params("http://www.reddit.com/hot.json", &[("limit", "1")]).unwrap();
        let response_json = json::from_str(response.body.as_slice()).unwrap();
        assert!(response_json.find(&"data".to_string()).unwrap().find(&"children".to_string()).unwrap().as_array().unwrap().len() == 1);
    }
}
