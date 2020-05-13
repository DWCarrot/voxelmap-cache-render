
mod color;
mod render;
mod application;
mod tilegen;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::App;
use clap::Arg;
use clap::SubCommand;
use slog::Logger;
use slog::Drain;
use slog_term::PlainSyncDecorator;
use slog_term::FullFormat;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");


const MAX_THREAD: usize = 16;

fn main() {

    let decoder = PlainSyncDecorator::new(io::stdout());
    let drain = FullFormat::new(decoder).build().fuse();
    let logger = Logger::root(drain, slog::o!());

    let app = 
        App::new("voxelmap cache offline renderer")
        .version(VERSION)
        .author(AUTHORS)
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
        )
        .subcommand(
            SubCommand::with_name("tile")
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
                Arg::with_name("filter")
                .long("filter")
                .help("filter used in scale, can be \"nearest\", \"triangle\", \"gaussian\", \"catmullrom\", \"lanczos3\"; default is \"nearest\"")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("path_mode")
                .long("path_mode")
                .help("generated path mode, can be \"layer:<start>,<step>,<stop>\"")
                .long_help("generated path mode, can be \"layer:<start>,<step>,<stop>\"\nexample: layer mode, the original scale is 5, finished at 0 => \"layer:5,-1,0\"")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("use_multi_thread")
                .long("use_multi_thread")
                .help("if use multi-thread; fixed 4 thread")
                .takes_value(false)
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
                let mut options = application::AppOptions::default();
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
            
            let app = Arc::new(application::Application::new(options));
            let time = Instant::now();
            let list = app.list_files();
            application::Application::alloc_tasks(app, list, &logger);
            let time = Instant::now() - time;
            slog::info!(logger, "> used {}ms", time.as_millis());
            
        },

        "tile" => {
            let options = {
                let input_folder = PathBuf::from(args.value_of("input_dir").unwrap());
                let output_folder = PathBuf::from(args.value_of("output_dir").unwrap());
                let mut options = tilegen::TileGeneratorOptions::new(input_folder, output_folder);
                if let Some(value) = args.value_of("filter") {
                    options.set_filter(value);
                }
                if args.is_present("use_multi_thread") {
                    options.set_multi_thread_mode(true);
                }
                options
            };
            let path_gen = options.build_path_generator(args.value_of("path_mode").unwrap()).unwrap();
            let app = tilegen::TileGenerator::new(options, path_gen);
            let time = Instant::now();
            let list = app.list_files();
            app.generate_tile(list, &logger);
            let time = Instant::now() - time;
            slog::info!(logger, "> used {}ms", time.as_millis());
        }

        _ => {

        }
    }
}


