use std::path::PathBuf;

use media::MediaPath;
use rocket::{fs::NamedFile, get, http::Status, launch, routes, State};
use rocket_dyn_templates::{context, Template};

mod media;

#[get("/")]
fn hello(base: &State<MediaPath>) -> Result<Template, (Status, String)> {
    let files = base.media_list();

    Ok(Template::render("index", context!(videos: files)))
}

#[get("/<path..>")]
async fn fetch_file(base: &State<MediaPath>, path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(base.base_dir().join(path)).await.ok()
}

#[get("/play/<file>")]
fn player(file: PathBuf) -> Template {
    Template::render("player", context!(video: file))
}

#[launch]
fn rocket() -> _ {
    let media_path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_string()));

    rocket::build()
        .manage(MediaPath::new(media_path))
        .mount("/", routes![hello, fetch_file, player])
        .attach(Template::fairing())
}
