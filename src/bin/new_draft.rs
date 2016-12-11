extern crate buprestidae;
extern crate dotenv;

use std::io::{stdin, Read};

fn main() {
    dotenv::dotenv().ok();

    println!("Title: ");
    let mut title = String::new();
    stdin().read_line(&mut title).unwrap();
    let title = &title[..(title.len() - 1)]; // drop newline

    println!("Body:\n");
    let mut body = String::new();
    stdin().read_to_string(&mut body).unwrap();

    let post = buprestidae::new_draft(&title, &body).unwrap();

    println!("Draft saved for {} ({}).", post.title, post.id);
}