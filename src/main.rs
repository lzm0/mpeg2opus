use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Deserialize, Serialize)]
struct AudioData {
    data: String,
}

fn convert(input_data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut cmd = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .arg("-c:a")
        .arg("libopus")
        .arg("-f")
        .arg("ogg")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = cmd
        .stdin
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to open stdin for ffmpeg"))?;

    let mut stdout = cmd
        .stdout
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to open stdout for ffmpeg"))?;

    let writer = thread::spawn(move || {
        stdin.write_all(&input_data).unwrap();
    });

    let mut output_data = Vec::new();
    stdout.read_to_end(&mut output_data)?;

    writer.join().unwrap();

    let exit_status = cmd.wait()?;

    if !exit_status.success() {
        return Err(Error::new(ErrorKind::Other, "ffmpeg failed"));
    }

    Ok(output_data)
}

async fn handle_post(body: web::Json<AudioData>) -> HttpResponse {
    let input_data = match base64::decode(&body.data) {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().body("Payload is not base64 encoded"),
    };
    let output_data = match convert(input_data) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    let output_str = base64::encode(&output_data);
    HttpResponse::Ok().json(AudioData { data: output_str })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(handle_post)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
