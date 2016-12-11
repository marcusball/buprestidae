extern crate buprestidae;
extern crate dotenv;

fn main() {
    dotenv::dotenv().ok();

    let posts = buprestidae::get_posts().unwrap();

    for post in posts {
        println!("{}\n-----\n{}", post.title, post.body);
    }
}