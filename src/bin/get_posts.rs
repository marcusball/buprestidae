extern crate buprestidae;
extern crate dotenv;

use buprestidae::models::Post;

fn main() {
    dotenv::dotenv().ok();

    let posts = buprestidae::get_posts().unwrap();

    for post in posts {
        println!("{}\n-----\n{}", post.title, post.body);
    }
}