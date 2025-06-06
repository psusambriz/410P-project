// templates.rs
use crate::quote::Quote;
use askama::Template; 


#[derive(Template)]
#[template(path = "index.html")] 
pub struct IndexTemplate {
    pub quote: Quote,
    pub stylesheet: &'static str,
}

impl IndexTemplate {
    pub fn new(quote: Quote) -> Self 
    {
        Self {
            quote,
            stylesheet: "/style.css",
        }
    }
}