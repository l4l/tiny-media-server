use std::path::PathBuf;

use media::MediaPath;
use rocket::{get, http::Status, launch, routes, State};
use rocket_include_tera::{
    tera_resources_initialize, tera_response, TeraContextManager, TeraResponse,
};

macro_rules! count {
    () => (0usize);
    ( $a:expr ) => (1usize);
    ( $x:expr, $($xs:expr),* ) => (1usize + count!( $($xs),* ));
}

macro_rules! hm {
    ($($k:expr => $v:expr),*) => {{
        let mut map = ::std::collections::HashMap::with_capacity(
            count!( $($k),* )
        );
        $( map.insert($k, $v); )*
        map
    }};
}

mod ffmpeg;
mod media;

#[get("/")]
fn hello(
    base: &State<MediaPath>,
    tera_cm: &State<TeraContextManager>,
) -> Result<TeraResponse, (Status, String)> {
    let files = base.media_list();

    Ok(tera_response!(
        tera_cm,
        Default::default(),
        "index",
        hm!("videos" => files)
    ))
}

#[get("/<path..>")]
async fn fetch_file(
    base: &State<MediaPath>,
    path: PathBuf,
) -> Result<ffmpeg::FfmpegStream, (Status, String)> {
    let path = base.base_dir().join(path);
    if !path.exists() {
        return Err((Status::NotFound, "file doesn't exist".into()));
    }

    ffmpeg::FfmpegStream::from_file(path).map_err(|e| (Status::InternalServerError, e))
}

#[get("/play/<file>")]
fn player(file: PathBuf, tera_cm: &State<TeraContextManager>) -> TeraResponse {
    tera_response!(tera_cm, Default::default(), "player", hm!("video" => file))
}

#[launch]
fn rocket() -> _ {
    ffmpeg::check_ffmpeg_present();

    let media_path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_string()));

    rocket::build()
        .manage(MediaPath::new(media_path))
        .mount("/", routes![hello, fetch_file, player])
        .attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "index" => "templates/index.html.tera",
                "player" => "templates/player.html.tera",
            );
        }))
}
