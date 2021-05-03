extern crate clap;
use clap::{App, Arg};
extern crate futures;
extern crate reqwest;
use futures::future::TryFutureExt;
use futures::future::FutureExt;
extern crate serde_json;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate tokio;
extern crate serde;
use serde::{Serialize, Serializer};
use crate::serde::ser::SerializeStruct;
extern crate chrono;
use chrono::prelude::*;
use std::io::{stdout, Write};

#[derive(Serialize)]
struct Product {
    identifier: String,
    version: String,
    arch: String,
}

#[derive(Serialize)]
enum Subscription {
    Active,
    Inactive,
}

#[derive(Serialize)]
struct Registration {
    regcode: String,
    starts_at: DateTime<Utc>, //TODO is the default rfc3339 format ok?
    subscription_status: Subscription,
}

struct StatusItem {
    product: Product,
    status: Option<Registration>,
}

impl Serialize for StatusItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let number_of_fields = 3 + 1 + if self.status.is_some() { 3 } else { 0 };
        let mut structure = serializer.serialize_struct("StatusItem", number_of_fields)?;

        structure.serialize_field("identifier", &self.product.identifier)?;
        structure.serialize_field("version", &self.product.version)?;
        structure.serialize_field("arch", &self.product.arch)?;

        match &self.status {
            None => structure.serialize_field("status", "Not registered")?,
            Some(r) => {
                structure.serialize_field("status", "Registered")?;
                structure.serialize_field("regcode", &r.regcode)?;
                structure.serialize_field("starts_ad", &r.starts_at)?;
                structure.serialize_field("subscription_status", &r.subscription_status)?;
            }
        }
        structure.end()
    }
}

type Status = Vec<StatusItem>;


fn status_from_system() -> Status {
    //TODO
    vec![ StatusItem{
        product: Product {
            identifier: "SLES".to_string(),
            version: "15.2".to_string(),
            arch: "x86_64".to_string(),
        },
        status: None,
    }]
}

async fn status_json() -> String {
    let mut result = status_from_system();
    for i in 0..result.len() {
        //TODO
        let registration = Registration{
            regcode: "testIGNORE".to_string(),
            starts_at: Utc::now(),
            subscription_status: Subscription::Active,
        };
        result[i].status = Some(registration);
    }
    serde_json::to_string(&result).unwrap()
}

async fn json(url: &str) -> String {
    let body = reqwest::get(url)
        .and_then(|response| response.text())
        .await
        .expect("HTTP request for json failed");

    let json: serde_json::Value = serde_json::from_str(&body).expect("Parsing json failed");
    format!("First crate is: {:#?}\n", json["crates"][0])
}

async fn xml(url: &str) -> String {
    let body = reqwest::get(url)
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

    format!("Width on root node is: {:#?}\n", value)
}

#[tokio::main]
async fn main() {
    let argument_matches = App::new("suseconnectr")
        .arg(
            Arg::with_name("status")
                .long("status")
                .help("Get current system registration status in json format.")
        )
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


    let mut out = Vec::new();

    if let Some(url) = argument_matches.value_of("json") {
        out.push(json(url).boxed());
    }
    if let Some(url) = argument_matches.value_of("xml") {
        out.push(xml(url).boxed());
    }
    if argument_matches.is_present("status") {
        out.push(status_json().boxed());
        out.push( async { "\n".to_string() }.boxed() );
    } else {
        out.push( async { "Done!\n".to_string() }.boxed() );
    }

    for future in out {
        stdout().write_all(future.await.as_bytes()).unwrap();
    }
}
