use std::path::PathBuf;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicUsize;

use bytes::Bytes;
use bytes::BytesMut;
use bytes::BufMut;
use actix_rt::System;
use actix_web::middleware;
use actix_web::web;
use actix_web::HttpServer;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::Error as ActixError;
use actix_web::http::header;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures::StreamExt;
use futures::TryStreamExt;
use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::Visitor;
use serde::de::MapAccess;

use super::render::RenderOptions;
use super::render;
use super::render::Tile;
use super::color::BakedColorManager;
use super::application;



const MAX_TILE_SIZE: usize = 256 * 1024;

#[derive(Clone)]
pub struct RenderServerOptions {
    host: String,
    workers: usize,
    max_tasks: usize,
    compress: bool,
}

impl Default for RenderServerOptions {

    fn default() -> Self {
        RenderServerOptions {
            host: String::from("0.0.0.0:8080"),
            workers: num_cpus::get(),
            max_tasks: 128,
            compress: false,
        }
    }
}

impl RenderServerOptions {

    pub fn set_host(&mut self, host: &str) {
        self.host = String::from(host);
    }

    pub fn set_workers(&mut self, num: usize) {
        self.workers = num;
    }

    pub fn set_max_tasks(&mut self, num: usize) {
        self.max_tasks = num;
    }

    pub fn set_compress(&mut self, compress: bool) {
        self.compress = compress;
    }
}


pub struct RenderService {
    options: RenderServerOptions,
    colormgr: Arc<BakedColorManager>,
    working: AtomicUsize,
}

impl RenderService {

    pub fn new(options: RenderServerOptions) -> Self {
        RenderService {
            colormgr: Arc::new(application::build_colormanager()),
            working: AtomicUsize::new(0),
            options
        }
    }

    pub fn start(self) {
        System::builder()
            .name("RenderService")
            .build()
            .block_on(run_service(self))
            .unwrap();
    }

}


impl<'de> Deserialize<'de> for RenderOptions {
    
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;
        use serde::de;

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = RenderOptions;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RenderOptions")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = RenderOptions::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "gamma" => {
                            res.set_gamma(map.next_value()?)
                        },
                        "light" => {
                            res.set_env_light(map.next_value()?)
                        },
                        _ => {
                            return Err(de::Error::unknown_field(key.as_str(), FIELDS));
                        }
                    }
                }
                Ok(res)
            }
        }

        const FIELDS: &'static [&'static str] = &["light", "gamma"];
        deserializer.deserialize_struct("RenderOptions", FIELDS, InnerVisitor)
    }
}


async fn run_service(service: RenderService) -> std::io::Result<()> {

    const ACTIX_LOG_FORMAT: &'static str = "%a \"%r\" %s \"%{User-Agent}i\" %D";

    let options = service.options.clone();
    let service = web::Data::new(service);
    let mut webfileroot = application::curdir();
    webfileroot.push("web");
    let webfileroot = web::Data::new(webfileroot);
    let payloadcfg = web::PayloadConfig::new(MAX_TILE_SIZE * 3 / 2);
    if options.compress {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::new(ACTIX_LOG_FORMAT))
                .wrap(middleware::Compress::default())
                .app_data(payloadcfg.clone())
                .app_data(webfileroot.clone())
                .app_data(service.clone())
                .service(
                    web::resource("/render")
                        .route(web::post().to(render))
                )
                .service(
                    web::resource("/")
                        .route(web::get().to(index_html))
                )
                .service(
                    web::resource("/index.html")
                        .route(web::get().to(index_html))
                )
                .service(
                    web::resource("/index.js")
                        .route(web::get().to(index_js))
                )
        })
        .bind(options.host)?
        .workers(options.workers)
        .run()
        .await
    } else {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::new(ACTIX_LOG_FORMAT))
                .app_data(payloadcfg.clone())
                .app_data(webfileroot.clone())
                .app_data(service.clone())
                .service(
                    web::resource("/render")
                        .route(web::post().to(render))
                )
                .service(
                    web::resource("/")
                        .route(web::get().to(index_html))
                )
                .service(
                    web::resource("/index.html")
                        .route(web::get().to(index_html))
                )
                .service(
                    web::resource("/index.js")
                        .route(web::get().to(index_js))
                )
        })
        .bind(options.host)?
        .workers(options.workers)
        .run()
        .await
    }
}


async fn index_html(root: web::Data<PathBuf>) -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(root.join("index.html"))?)
}


async fn index_js(root: web::Data<PathBuf>) -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open(root.join("index.js"))?)
}


async fn render(s: web::Data<RenderService>, query: web::Query<RenderOptions>, mut payload: Multipart) -> Result<HttpResponse, ActixError> {
    
    struct StringifyError(String);

    impl std::fmt::Debug for StringifyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.0.as_str())
        }
    }
    
    const EXT: &'static str = ".zip";

    let parse_tile_id = |filename: &str| -> Option<(i32, i32)> {
        if filename.ends_with(EXT) {
            let filename = &filename[0 .. filename.len() - EXT.len()];
            let mut sp = filename.splitn(2, ',');
            let x: i32 = sp.next()?.parse().ok()?;
            let z: i32 = sp.next()?.parse().ok()?;
            Some((x, z))
        } else {
            None
        }
    };

    if s.working.load(Ordering::SeqCst) >= s.options.max_tasks {
        return Ok(HttpResponse::TooManyRequests().into())
    }

    let render_options = query.into_inner();
    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
        
        s.working.fetch_add(1, Ordering::SeqCst);

        let mut tile_id = (0, 0);
        let mut some_tile_id = None;
        if let Some(filename0) = content_type.get_filename() {
            some_tile_id = parse_tile_id(filename0);
            if let Some(v) = &some_tile_id {
                tile_id = v.clone();
            }
        }
        let mut buf = BytesMut::with_capacity(MAX_TILE_SIZE);
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            if data.len() + buf.len() > MAX_TILE_SIZE {
                s.working.fetch_sub(1, Ordering::SeqCst);
                return Ok(
                    HttpResponse::PayloadTooLarge()
                        .into()
                );
            }
            buf.put(data);
        }
        let mgr = s.colormgr.clone();
        let r = web::block(move || -> Result<Bytes, String> {
            let ifile = Cursor::new(buf);
            let tile = Tile::load(ifile, tile_id, &mgr).map_err(|e| e.to_string())?;
            let pic = render::render(tile, &mgr, &render_options);
            let mut ofile = Vec::with_capacity((pic.width() * pic.height() * 4 / 3) as usize);
            image::DynamicImage::ImageRgba8(pic).write_to(&mut ofile, image::ImageFormat::Png).map_err(|e| e.to_string())?;
            Ok(Bytes::from(ofile))
        })
        .await;

        s.working.fetch_sub(1, Ordering::SeqCst);

        match r {
            Ok(buf) => {
                let mut builder = HttpResponse::Ok();
                builder.set(header::ContentType::png());
                if let Some((x, z)) = some_tile_id {
                    builder.set(header::ContentDisposition {
                        disposition: header::DispositionType::Attachment,
                        parameters: vec![
                            header::DispositionParam::Filename(format!("{},{}.png", x, z))
                        ]
                    });
                }
                let builder = builder.body(buf);
                return Ok(
                    builder.into()
                );
            }
            Err(e) => {
                return Ok(
                    HttpResponse::BadRequest()
                        .body(e.to_string())
                        .into()
                );
            }
        }
    }
    Ok(HttpResponse::NotFound().into())
}


mod test {

    use super::*;

    #[test]
    fn test_server() {
        std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
        env_logger::init();
        let options = RenderServerOptions::default();
        RenderService::new(options).start();
    }


}


