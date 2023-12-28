use crate::config::get_config;

mod config;

fn main() {
    let config = get_config(None);
    println!("{config:#?}")
}
