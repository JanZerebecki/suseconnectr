extern crate clap;
use clap::{App, Arg};
extern crate futures;
extern crate reqwest;
use futures::future::TryFutureExt;
extern crate serde_json;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate tokio;

#[tokio::main]
async fn main() {
    let argument_matches = App::new("suseconnectr")
        .arg(
            Arg::with_name("json")
                .long("json")
                .value_name("URL")
                .help("Try https://play.rust-lang.org/meta/crates")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("xml")
                .long("xml")
                .value_name("URL")
                .help("Try https://doc.rust-lang.org/brush1.51.0.svg")
                .takes_value(true),
        )
        .get_matches();

    if let Some(url) = argument_matches.value_of("json") {
        let body = reqwest::get(url.to_string())
            .and_then(|response| response.text())
            .await
            .expect("HTTP request for json failed");

        let json: serde_json::Value = serde_json::from_str(&body).expect("Parsing json failed");
        println!("First crate is: {:#?}", json["crates"][0])
    }

    if let Some(url) = argument_matches.value_of("xml") {
        let body = reqwest::get(url.to_string())
            .and_then(|response| response.text())
            .await
            .expect("HTTP request for xml failed");

        let package = sxd_document::parser::parse(&body).expect("Parsing xml failed");
        let xml = package.as_document();
        let mut context = sxd_xpath::Context::new();

        context.set_namespace("svg", "http://www.w3.org/2000/svg");
        let xpath_source = "/svg:svg/@width";

        let factory = sxd_xpath::Factory::new();
        let xpath = factory
            .build(xpath_source)
            .expect("Could not compile XPath")
            .expect("No XPath was compiled");
        let value = xpath
            .evaluate(&context, xml.root())
            .expect("XPath evaluation failed")
            .string();

        println!("Width on root node is: {:#?}", value)
    }

    println!("Done!");
}
