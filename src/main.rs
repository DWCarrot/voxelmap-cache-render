
mod color;
mod render;
mod application;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use clap::App;
use clap::Arg;
use clap::SubCommand;

use application::AppOptions;
use application::Application;


const MAX_THREAD: usize = 16;

fn main() {
    
    let app = 
        App::new("voxelmap cache offline renderer")
        .subcommand(
            SubCommand::with_name("render")
            .arg(
                Arg::with_name("input_dir")
                .short("i")
                .long("input_dir")
                .help("input folder")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("output_dir")
                .short("o")
                .long("output_dir")
                .help("output folder")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("env_light")
                .long("env_lit")
                .help("environment light, from 0 to 15, default is 15")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("gamma")
                .long("gamma")
                .help("gamma for gamma correction, default is 1.0")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("thread")
                .short("t")
                .long("thread")
                .help("multi-thread: thread number")
                .takes_value(true)
            )
        );

    let matches = app.get_matches();
    let (name, args) = matches.subcommand();
    let args = if let Some(v) = args {
        v
    } else {
        println!("{}", matches.usage());
        return;
    };
    match name {
        "render" => {
            let options = {
                let mut options = AppOptions::default();
                options.input_folder = PathBuf::from(args.value_of("input_dir").unwrap());
                options.output_folder = {
                    let f = PathBuf::from(args.value_of("output_dir").unwrap());
                    if !f.is_dir() {
                        std::fs::create_dir_all(f.as_path()).unwrap();
                    }
                    f
                };
                if let Some(gamma) = args.value_of("gamma") {
                    if let Ok(gamma) = gamma.parse() {
                        if gamma > 0.0 {
                            options.render_options.gamma = gamma;
                        }
                    }
                }
                if let Some(lit) = args.value_of("env_light") {
                    if let Ok(lit) = lit.parse() {
                        if lit < 15 {
                            options.render_options.env_light = lit;
                        }
                    } 
                }
                if let Some(thread) = args.value_of("thread") {
                    if let Ok(thread) = thread.parse() {
                        if thread <= MAX_THREAD {
                            options.thread_num = thread;
                        }
                    }
                }
                options
            };
            
            let app = Arc::new(Application::new(options));
            let list = app.list_files();
            Application::alloc_tasks(app, list);
            
        },
        _ => {

        }
    }
}


