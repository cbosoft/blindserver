use std::{fs::{create_dir_all, OpenOptions}, io::{Cursor, Read, Write}, path::PathBuf};

use axum::{
    body::Bytes, extract::{
        DefaultBodyLimit,
        Multipart
    }, routing::post, Router
};
use flate2::read::GzDecoder;
use tar::{Entry, EntryType};
use tower_http::services::ServeDir;


const KB: usize = 1024;
const MB: usize = KB*KB;
const GB: usize = MB*KB;


// :PORT options?
const PORT: u16 = 3000;

// DESTINATION options?
const DEST: &'static str = "payload";

// MAX FILE SIZE options?
const MAX_FILE_SIZE: usize = 100*MB;


fn open_archive(b: Bytes) {
    let c = Cursor::new(b);
    let decomp = GzDecoder::new(c);
    let mut archive = tar::Archive::new(decomp);
    let entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading .tar.gz archive: {e:?}");
            return;
        }
    };

    for e in entries {
        let mut e = match e {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error when trying to read entry: {e:?}.");
                return;
            }
        };

        let path = e.header().path().unwrap().to_path_buf();
        let size = e.header().size().unwrap();
        let dest_path = PathBuf::new().join(".").join(DEST).join(&path);

        match e.header().entry_type() {
            EntryType::Regular => {
                match e.unpack(dest_path.clone()) {
                    Err(e) => {
                        eprintln!("Failed to unpack entry {}: {e:?}", path.display());
                        continue;
                    }
                    Ok(_) => ()
                }
                eprintln!("{} -> {} ({size} bytes)", path.display(), dest_path.display());
            }
            EntryType::Directory => {
                match create_dir_all(dest_path.clone()) {
                    Err(e) => {
                        eprintln!("Failed to create dir(s) {}: {e:?}", path.display());
                        return;
                    }
                    Ok(_) => ()
                }
                eprintln!("{} -> {} (new dir)", path.display(), dest_path.display());
            }
            _ => ()
        }
    }
}


async fn upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();
        open_archive(data);
    }
}


#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/upload", post(upload))
        .nest_service("/docs", ServeDir::new(DEST))
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE));

    let path = format!("0.0.0.0:{PORT}");
    eprintln!("Listening on {path}");
    let listener = tokio::net::TcpListener::bind(path).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
