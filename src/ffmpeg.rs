use std::path::PathBuf;
use std::process::Stdio;

use rocket::{
    http::ContentType,
    response::{self, Responder},
    Response,
};
use tokio::process::{ChildStdout, Command};

pub fn check_ffmpeg_present() {
    let child = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "ffmpeg not found: {}",
        String::from_utf8_lossy(&child.stderr)
    );
}

pub struct FfmpegStream(ChildStdout);

impl FfmpegStream {
    pub fn from_file(path: PathBuf) -> Result<Self, String> {
        Command::new("ffmpeg")
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .arg("-i")
            .arg(path)
            .args(["-listen", "1"])
            .args(["-f", "mp4", "-movflags", "frag_keyframe+empty_moov"])
            .args(["pipe:1"])
            .spawn()
            .map_err(|e| e.to_string())?
            .stdout
            .take()
            .ok_or_else(|| "ffmpeg stdout closed".into())
            .map(Self)
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for FfmpegStream {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::MP4)
            .streamed_body(self.0)
            .ok()
    }
}
