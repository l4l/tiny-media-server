use std::path::{Path, PathBuf};
use std::process::Stdio;

use rocket::{
    http::ContentType,
    response::{self, Responder},
    Request, Response,
};
use tokio::{
    fs::File,
    process::{ChildStdout, Command},
};

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
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::MP4)
            .streamed_body(self.0)
            .ok()
    }
}

const THUMBNAIL_EXT: &str = "tms-thumb";

pub struct Thumbnail(File);

impl Thumbnail {
    pub async fn new(file_path: &Path) -> Option<std::io::Result<Self>> {
        let thumbnail_path = file_path.with_extension(THUMBNAIL_EXT);

        if !thumbnail_path.exists() {
            let duration = Self::file_duration(file_path).await;
            let thumbnail_time = (duration.unwrap_or(500.) / 2.).min(250.);
            Self::create(thumbnail_time, file_path, &thumbnail_path).await?;
        }

        Some(File::open(thumbnail_path).await.map(Self))
    }

    async fn file_duration(path: &Path) -> Option<f32> {
        let stdout = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
            ])
            .arg(&path)
            .output()
            .await
            .ok()?
            .stdout;
        std::str::from_utf8(&stdout).ok()?.trim().parse().ok()
    }

    async fn create(at: f32, file_path: &Path, thmb_path: &Path) -> Option<()> {
        Command::new("ffmpeg")
            .args(["-v", "error"])
            .arg("-ss")
            .arg(at.to_string())
            .arg("-i")
            .arg(file_path)
            .args([
                "-vf",
                "thumbnail,scale=300:200:force_original_aspect_ratio=decrease,pad=300:200:(ow-iw)/2:(oh-ih)/2",
                "-frames:v",
                "1",
                "-f",
                "image2",
                "-c",
                "png"
            ])
            .arg(thmb_path)
            .status()
            .await
            .ok()?
            .success()
            .then_some(())
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Thumbnail {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            .header(ContentType::PNG)
            .raw_header("Cache-control", "max-age=86400")
            .ok()
    }
}
