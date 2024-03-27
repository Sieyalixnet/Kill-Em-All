mod core;
mod render;
use core::selector::Selector;
use render::renderer::exit;
use std::io::stdout;

/// Search for a pattern in a file and display the lines that contain it.
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path that start to check dirs. If should contain many dirs.
   #[clap(short, long, value_parser, default_value = "./")]
    path:std::path::PathBuf
}


fn run(path:std::path::PathBuf){
    let mut stdout = stdout();
    let mut menu_ui = Selector::new(0);
    if menu_ui.init(path)==false{
        return;
    };
    loop {
        let res: usize = menu_ui.render(&mut stdout);
        if res == usize::MAX {
            exit(&mut stdout);
            return;
        } else {
            {
                menu_ui.render(&mut stdout);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    run(args.path);
}
