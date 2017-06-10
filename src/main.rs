#[macro_use]
extern crate askama; // for the Template trait and custom derive macro
extern crate iron;

use std::collections::HashMap;
use std::env;

use iron::prelude::*;
use iron::Handler;
use iron::status;
use iron::headers;
use iron::headers::{ContentDisposition, DispositionParam, Charset};
use iron::modifiers::Header;
use askama::Template;

const CORRECT_MIMES: bool = true;
const DOWNLOAD: bool = true;

const IMG1: &'static [u8] = include_bytes!("../res/chess-pattern.png");
const PDF1: &'static [u8] = include_bytes!("../res/pdf-sample.pdf");

const IMG1_URL: &'static str = "res/image1";
const PDF1_URL: &'static str = "res/pdf1";
const TEXT1_URL: &'static str = "res/text1";
const TEXT_HTML_URL: &'static str = "html";

const HTML: IndexTemplate = IndexTemplate {
    image_link: IMG1_URL,
    pdf_link: PDF1_URL,
    text_link: TEXT1_URL,
    html_link: TEXT_HTML_URL
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    image_link: &'a str,
    pdf_link: &'a str,
    text_link: &'a str,
    html_link: &'a str,
}

struct Router {
    // Routes here are simply matched with the url path.
    routes: HashMap<String, Box<Handler>>
}

impl Router {
    fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    fn add_route<H>(&mut self, path: String, handler: H) where H: Handler {
        self.routes.insert(path, Box::new(handler));
    }
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match self.routes.get(&req.url.path().join("/")) {
            Some(handler) => handler.handle(req),
            None => Ok(Response::with(status::NotFound))
        }
    }
}

fn create_content_disposition(filename: &[u8]) -> ContentDisposition {
    let content_disposition_type: headers::DispositionType =
        if DOWNLOAD { headers::DispositionType::Attachment }
        else { headers::DispositionType::Inline };
    ContentDisposition {
        disposition: content_disposition_type,
        parameters: vec![
            DispositionParam::Filename(
                Charset::Iso_8859_1, // The character set for the bytes of the filename
                None, // The optional language tag (see `language-tag` crate)
                filename.to_vec() // the actual bytes of the filename
            )
        ]
    }
}

fn main() {
    let mut router = Router::new();

    router.add_route("".to_string(), |_: &mut Request| {
        let html_string:  String = HTML.render();
        Ok(Response::with((
            status::Ok,
            html_string,
            Header(headers::ContentType::html())
        )))
    });

    router.add_route(IMG1_URL.to_string(), |_: &mut Request| {

        let content_type =
            if CORRECT_MIMES { headers::ContentType::png() }
            else { headers::ContentType("mytype/forimg".parse().unwrap()) };
        Ok(Response::with((
            status::Ok,
            IMG1,
            Header(content_type),
            Header(create_content_disposition(b"image.png"))
        )))
    });

    router.add_route(PDF1_URL.to_string(), |_: &mut Request| {
        let content_type = headers::ContentType(
            if CORRECT_MIMES { "application/pdf" }
            else { "mytype/forpdf" }
            .parse().unwrap()
        );
        Ok(Response::with((
            status::Ok,
            PDF1,
            Header(content_type),
            Header(create_content_disposition(b"sample-pdf.pdf"))
        )))
    });

    router.add_route(TEXT1_URL.to_string(), |_: &mut Request| {
        let content_type = headers::ContentType(
            if CORRECT_MIMES { "text/plain" }
            else { "mytype/fortext" }
            .parse().unwrap()
        );
        Ok(Response::with((
            status::Ok,
            "neki text 1",
            Header(content_type),
            Header(create_content_disposition(b"text.txt"))
        )))
    });

    router.add_route(TEXT_HTML_URL.to_string(), |_: &mut Request| {
        let html_string:  String = HTML.render();
        let content_type =
            if CORRECT_MIMES { headers::ContentType::html() }
            else { headers::ContentType("mytype/forhtml".parse().unwrap()) };
        Ok(Response::with((
            status::Ok,
            html_string,
            Header(content_type),
            Header(create_content_disposition(b"page.html"))
        )))
    });

    router.add_route("error".to_string(), |_: &mut Request| {
        Ok(Response::with(status::BadRequest))
    });

    let args: Vec<String> = env::args().collect();
    let host: String = args.get(1).map(|s| s.to_owned()).unwrap_or("0.0.0.0".to_string());
    let port: String = args.get(2).map(|s| s.to_owned()).unwrap_or("3000".to_string());
    let addr = format!("{}:{}", host, port);
    println!("Serving at {}", addr);
    Iron::new(router).http(addr).unwrap();
}
