
mod color;
mod render;
mod application;
mod tilegen;

#[cfg(feature = "service")]
mod service;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::str::FromStr;

use clap::App;
use clap::Arg;
use clap::SubCommand;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");


const MAX_THREAD: usize = 16;

fn main() {

    if let Err(_e) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let app = 
        App::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .author(AUTHORS);
    let app = app.subcommand(
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
    let app = app.subcommand(
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
                .help("generated path mode, can be \"layer+\", \"layer+:<minZoom>\", \"layer+:<minZoom>,<maxZoom>\", \"layer-\", \"layer-:<minZoom>\", \"layer-:<maxZoom>,<minZoom>\"")
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
    #[cfg(feature = "service")]
    let app = app.subcommand(
            SubCommand::with_name("renderserver")
            .arg(
                Arg::with_name("host")
                .long("host")
                .help("server bind host; default is \"0.0.0.0:8080\"")
                .takes_value(true)
                .required(false)
            )
            .arg(
                Arg::with_name("max_tasks")
                .long("max_tasks")
                .help("max tasks number for server to run at the same time; default is 128")
                .takes_value(true)
                .required(false)
            )
            .arg(
                Arg::with_name("workers")
                .long("workers")
                .help("worker thread num for server")
                .takes_value(true)
                .required(false)
            )
            .arg(
                Arg::with_name("compress")
                .long("compress")
                .help("to enable compress for server")
                .required(false)
            )
            .arg(
                Arg::with_name("tls")
                .long("tls")
                .help("use tls with specific cert.pem and key.pem; format: --tls path-to-cert.pem,path-to-key.pem")
                .takes_value(true)
                .required(false)
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
                options.set_input_folder(args.value_of("input_dir").unwrap());
                options.set_output_folder(args.value_of("output_dir").unwrap());
                options.ensure_output_folder().unwrap();
                if let Some(gamma) = args.value_of("gamma") {
                    if let Ok(gamma) = gamma.parse() {
                        options.render_option_mut().set_gamma(gamma);
                    }
                }
                if let Some(lit) = args.value_of("env_light") {
                    if let Ok(lit) = lit.parse() {
                        options.render_option_mut().set_env_light(lit);
                    } 
                }
                if let Some(thread) = args.value_of("thread") {
                    if let Ok(thread) = thread.parse() {
                        if thread <= MAX_THREAD {
                            options.set_thread_num(thread);
                        }
                    }
                }
                options
            };
            
            let app = Arc::new(application::Application::new(options));
            let time = Instant::now();
            let list = app.list_files();
            application::Application::alloc_tasks(app, list);
            let time = Instant::now() - time;
            log::info!("> used {}ms", time.as_millis());
            
        },

        "tile" => {
            let options = {
                let input_folder = PathBuf::from(args.value_of("input_dir").unwrap());
                let output_folder = PathBuf::from(args.value_of("output_dir").unwrap());
                let path_mode = tilegen::PathMode::from_str(args.value_of("path_mode").unwrap()).unwrap();
                let mut options = tilegen::TileGeneratorOptions::new(input_folder, output_folder, path_mode);
                if let Some(value) = args.value_of("filter") {
                    options.set_filter(value);
                }
                if args.is_present("use_multi_thread") {
                    options.set_multi_thread_mode(true);
                }
                options
            };
            let app = tilegen::TileGenerator::new(options);
            let time = Instant::now();
            let list = app.list_files();
            app.generate_tile(list);
            let time = Instant::now() - time;
            log::info!("> used {}ms", time.as_millis());
        }

        #[cfg(feature = "service")]
        "renderserver" => {
            let options = {
                let mut options = service::RenderServerOptions::default();
                if let Some(host) = args.value_of("host") {
                    options.set_host(host);
                }
                if let Some(max_tasks) = args.value_of("max_tasks") {
                    if let Ok(num) = max_tasks.parse() {
                        options.set_max_tasks(num);
                    }
                }
                if let Some(workers) = args.value_of("workers") {
                    if let Ok(num) = workers.parse() {
                        options.set_workers(num);
                    }
                }
                options.set_compress(
                    args.is_present("compress")
                );
                if let Some(tls) = args.value_of("tls") {
                    let a: Vec<_> = tls.splitn(2, ',').collect();
                    if a.len() == 2 {
                        options.set_tls(PathBuf::from(a[0]), PathBuf::from(a[1]));
                    }
                }
                options
            };
            service::RenderService::new(options).start();
        }

        _ => {
            unimplemented!()
        }
    }
}


