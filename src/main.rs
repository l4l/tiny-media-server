#[macro_use]
extern crate rocket;

use std::path::{Path, PathBuf};

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn hello(base: &State<MediaPath>) -> Result<Template, (Status, String)> {
    let base_dir = base.base_dir();
    let files = walkdir::WalkDir::new(base_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().strip_prefix(&base_dir).unwrap().to_owned())
        .collect::<Vec<_>>();

    Ok(Template::render("index", context!(videos: files)))
}

struct MediaPath(Option<PathBuf>);

impl MediaPath {
    fn base_dir(&self) -> &Path {
        self.0.as_deref().unwrap_or_else(|| Path::new("."))
    }
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
    let media_path = std::env::args().nth(1);

    rocket::build()
        .manage(MediaPath(media_path.map(Into::into)))
        .mount("/", routes![hello, fetch_file, player])
        .attach(Template::fairing())
}
