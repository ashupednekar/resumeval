use askama::Template;
use serde::Serialize;

use crate::pkg::internal::project::Project;

#[derive(Template)]
#[template(path = "buckets.html")]
pub struct Buckets<'a> {
    pub username: &'a str,
}

#[derive(Template)]
#[template(path = "containers.html")]
pub struct Containers<'a> {
    pub username: &'a str,
}

#[derive(Template)]
#[template(path = "functions.html")]
pub struct Functions<'a> {
    pub username: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub containers: i32,
    pub functions: i32,
    pub buckets: i32,
    pub total_requests: i32,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct Home<'a> {
    pub username: &'a str,
    pub projects: Vec<Project>,
    pub metrics: Metrics,
}

#[derive(Template)]
#[template(path = "verify.html")]
pub struct Verify {}
