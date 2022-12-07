use actix_web::{web, App, HttpResponse, HttpServer};
use base64::decode;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::process::{Command, Stdio};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(handle_post)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[derive(Deserialize, Serialize)]
struct AudioData {
    data: String,
}

async fn handle_post(body: web::Json<AudioData>) -> HttpResponse {
    let input_data = decode(&body.data).unwrap();
    let output_data = convert(input_data);
    let output_str = base64::encode(&output_data);
    HttpResponse::Ok().json(AudioData { data: output_str })
}

fn convert(input_data: Vec<u8>) -> Vec<u8> {
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
        .spawn()
        .unwrap();

    cmd.stdin.as_mut().unwrap().write_all(&input_data).unwrap();

    let output = cmd.wait().unwrap();
    assert!(output.success());

    let mut output_data = Vec::new();
    cmd.stdout
        .as_mut()
        .unwrap()
        .read_to_end(&mut output_data)
        .unwrap();

    output_data
}
