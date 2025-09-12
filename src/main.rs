use std::path::Path;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use futures_util::StreamExt;
use rand::{Rng, distr::Alphanumeric};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

const GAMES_FILE: &str = "./data/games.json";
const UPLOADS_DIR: &str = "./data/uploads";
const STATIC_DIR: &str = "./data/static";

#[actix_web::get("/games")]
async fn games_json() -> impl Responder {
    match tokio::fs::read_to_string(GAMES_FILE).await {
        Ok(r) => match serde_json::from_str::<Value>(&r) {
            Ok(l) => serde_json::to_string(&l).unwrap_or("[]".to_string()),
            Err(_) => "[]".to_string(),
        },
        Err(e) => {
            eprintln!("Failed to read games file! {e}");
            "[]".to_string()
        }
    }
}

#[actix_web::post("/games")]
async fn add_game(mut payload: actix_multipart::Multipart) -> impl Responder {
    let mut meta: Option<Value> = None;
    let filename: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let upload_path = format!("{}/{}.zip", UPLOADS_DIR, filename);
    let mut file = match tokio::fs::File::create(&upload_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create upload file: {e}");
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to store upload"}));
        }
    };

    while let Some(Ok(mut field)) = payload.next().await {
        let name = field.name().unwrap_or("");
        if name == "file" {
            while let Some(Ok(chunk)) = field.next().await {
                if let Err(e) = file.write_all(&chunk).await {
                    eprintln!("Error writing upload chunk: {e}");
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Failed writing file"}));
                }
            }
        } else if name == "meta" {
            let mut buf = Vec::new();
            while let Some(Ok(chunk)) = field.next().await {
                buf.extend_from_slice(&chunk);
            }
            match String::from_utf8(buf)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            {
                Some(v) => meta = Some(v),
                None => {
                    eprintln!("Invalid metadata JSON received in multipart");
                    return HttpResponse::BadRequest().json(json!({"error": "Invalid metadata"}));
                }
            }
        } else {
            while let Some(Ok(_)) = field.next().await {}
        }
    }

    if let Err(e) = file.flush().await {
        eprintln!("Error flushing uploaded file: {e}");
        return HttpResponse::InternalServerError()
            .json(json!({"error": "Failed finalizing upload"}));
    }
    drop(file);

    std::fs::create_dir_all(format!("{}/play/{filename}", STATIC_DIR)).unwrap();

    let fname = filename.clone();
    let unzip_result =
        tokio::task::spawn_blocking(move || -> Result<(String, String), std::io::Error> {
            let f = std::fs::File::open(&upload_path)?;
            let mut zip =
                zip::ZipArchive::new(f).map_err(|e| std::io::Error::other(e.to_string()))?;

            let mut index_rel_dir: Option<String> = None;
            let mut img_rel_path: Option<String> = None;

            for ind in 0..zip.len() {
                let mut entry = zip.by_index(ind).map_err(|e| std::io::Error::other(e.to_string()))?;
                let Some(rel_path) = entry.enclosed_name().map(|p| p.to_path_buf()) else { continue; };
                let mut comps = rel_path.components();
                let _ = comps.next();
                let flattened: std::path::PathBuf = comps.collect();
                let path = Path::new(&format!("{}/play/{filename}/", STATIC_DIR)).join(&flattened);

                if entry.name().contains("__MACOSX") {
                } else if entry.name().ends_with('/') {
                    std::fs::create_dir_all(path.display().to_string().trim_end_matches("/"))?;
                } else {
                    if let Some(p) = path.parent() {
                        std::fs::create_dir_all(format!("{}", p.display()))?;
                    }
                    let mut outfile = std::fs::File::create(&path)?;
                    std::io::copy(&mut entry, &mut outfile)?;

                    if let Some(fname) = flattened.file_name().and_then(|s| s.to_str()) {
                        if fname.eq_ignore_ascii_case("index.html") {
                            let dir = flattened.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "".to_string());
                            index_rel_dir = Some(dir);
                        } else if fname.eq_ignore_ascii_case("image.png") {
                            let rel = flattened.to_string_lossy().to_string();
                            img_rel_path = Some(rel);
                        }
                    }
                }
            }

            let Some(index_dir) = index_rel_dir else { return Err(std::io::Error::other("Zip file does not contain index.html")); };
            let Some(img_rel) = img_rel_path else { return Err(std::io::Error::other("Zip file does not contain image.png")); };

            Ok((index_dir, img_rel))
        })
        .await;

    let base_web_path = format!("/play/{fname}/");
    let img_web_path = format!("/play/{fname}/image.png");

    let game_data = match meta {
        Some(mut v) => {
            if let Some("6767") = v["pw"].as_str() {
            } else {
                return HttpResponse::BadRequest()
                    .json(json!({"error": "Incorrect or missing password"}));
            }
            v["path"] = json!(base_web_path);
            v["img"] = json!(img_web_path);
            v
        }
        None => json!({}),
    };

    let games = match tokio::fs::read_to_string(GAMES_FILE).await {
        Ok(r) => match serde_json::from_str::<Value>(&r) {
            Ok(mut l) => match l.as_array_mut() {
                Some(o) => {
                    if let Some(name) = game_data.get("name").and_then(|n| n.as_str()) {
                        if let Some(old) = o.iter().find(|g| g.get("name").and_then(|n| n.as_str()) == Some(name)) {
                            if let Some(old_path) = old.get("path").and_then(|p| p.as_str()) {
                                let abs_path = format!("{}{}", STATIC_DIR, old_path);
                                if let Err(e) = std::fs::remove_dir_all(&abs_path) {
                                    eprintln!("Failed to delete old game folder {}: {}", abs_path, e);
                                }
                            }
                        }
                        o.retain(|g| g.get("name").and_then(|n| n.as_str()) != Some(name));
                    }
                    o.push(game_data);
                    o.clone()
                }
                None => vec![game_data],
            },
            Err(_) => vec![game_data],
        },
        Err(e) => {
            eprintln!("Failed to read games file! {e}");
            vec![game_data]
        }
    };

    match tokio::fs::write(GAMES_FILE, serde_json::to_string(&games).unwrap()).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing to games file: {e}");
        }
    };
    HttpResponse::Ok().json(json!({"status": "Success"}))
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    tokio::fs::create_dir_all(UPLOADS_DIR).await?;
    tokio::fs::create_dir_all(STATIC_DIR).await?;
    println!("Running on http://0.0.0.0:8080/");
    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::default().limit(100 * 1024 * 1024))
            .service(games_json)
            .service(add_game)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
            .service(actix_files::Files::new("/play", format!("{}/play", STATIC_DIR)).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}