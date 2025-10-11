use askama::Template;

use crate::pkg::internal::project::Project;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Home<'a> {
    pub username: &'a str,
    pub projects: Vec<Project>,
}

#[derive(Template)]
#[template(path = "verify.html")]
pub struct Verify {}
