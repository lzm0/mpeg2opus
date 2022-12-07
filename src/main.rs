use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, Error, ErrorKind};
use std::process::{Command, Stdio};

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

    let stdin = cmd
        .stdin
        .as_mut()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to open stdin for ffmpeg"))?;

    stdin.write_all(&input_data)?;

    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        return Err(Error::new(ErrorKind::Other, "ffmpeg failed"));
    }

    let mut output_data = Vec::new();
    output.stdout.take(1000000).read_to_end(&mut output_data)?;

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
