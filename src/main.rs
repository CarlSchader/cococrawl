use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directories to crawl positional arguments
    #[clap(required = true)]
    directories: Vec<String>,
}

fn main() {
    let args = Args::parse();
    println!("Crawling directories: {:?}", args.directories);
}
