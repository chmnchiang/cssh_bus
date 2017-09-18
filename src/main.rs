#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;

extern crate reqwest;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate serde_yaml;

extern crate log;

use log::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use serde_json::Value;

extern crate rocket;
extern crate rocket_contrib;
use rocket::response;
use rocket_contrib::Json;

#[derive(Serialize, Deserialize, Debug)]
struct Route {
    rid: i64,
    sec: i64,
    name: String,
    to: String,
    stop_id: usize,
    stop_name: String,
}

#[derive(Serialize, Debug)]
struct RouteData {
    route: &'static Route,
    eta: i64,
}

type FetchResult = Result<RouteData, String>;

fn proc_route(route: &'static Route) -> FetchResult {
    const BASE_URL: &'static str = "http://routes.5284.com.tw/ntpcebus/Js/RouteInfo";
    let url = format!("{}?rid={}&sec={}", BASE_URL, route.rid, route.sec);

    let resp = reqwest::get(&*url).map_err(|_| "Get result build failed")?;
    if !resp.status().is_success() {
        return Err(format!("Return status {}", resp.status()));
    }

    let json_data: Value = serde_json::from_reader(resp).map_err(|_| "Parse json failed")?;
    let eta = json_data.get("Etas")
        .and_then(|x| x.get(route.stop_id))
        .and_then(|x| x.get("eta"))
        .and_then(|x| x.as_i64())
        .ok_or("Extract value from json failed")?;

    println!("{}: eta = {}", route.name, eta);

    Ok(RouteData{
        route: route,
        eta: eta,
    })
}

lazy_static! {
    static ref ROUTE_YAML: Vec<Route> = {
        let file = File::open("data.yaml")
            .map_err(|_| "Open data.yaml failed").unwrap();

        let yaml_data: Vec<Route> = serde_yaml::from_reader(file)
            .map_err(|_| "yaml parsed failed").unwrap();

        yaml_data
    };
}

#[get("/")]
fn index() -> Option<response::NamedFile> {
    files("index.html".into())
}

#[get("/<path..>", rank=5)]
fn files(path: PathBuf) -> Option<response::NamedFile> {
    response::NamedFile::open(Path::new("static/").join(path)).ok()
}



#[get("/routes")]
fn get_routes() -> Result<Json<Vec<RouteData>>, response::status::Custom<&'static str>> {
    let result: Result<Vec<RouteData>, String> = ROUTE_YAML.iter().map(proc_route).collect();
    match result {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(response::status::Custom(rocket::http::Status::InternalServerError, "Can't fetch data"))
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![files, index])
        .mount("/ajax/", routes![get_routes])
        .launch();
}
