extern crate buprestidae;
extern crate dotenv;

use std::io::{stdin, Read};

fn main() {
    dotenv::dotenv().ok();

    println!("Title: ");
    let mut title = String::new();
    stdin().read_line(&mut title).unwrap();
    let title = title.replace(|c| c == '\r' || c == '\n', "");

    println!("Body:");
    let mut body = String::new();
    stdin().read_line(&mut body).unwrap();
    let body = body.replace(|c| c == '\r' || c == '\n', "");

    let post = buprestidae::new_draft(&title, &body).unwrap();

    println!("\nDraft saved for {} ({:?}) :D", post.title, post.id);
}