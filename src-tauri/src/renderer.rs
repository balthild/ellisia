use std::io::Cursor;
use std::str::FromStr;
use std::sync::LazyLock;

use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, ImageOutputFormat};
use rayon_core::ThreadPoolBuilder;
use regex::Regex;
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use typed_path::Utf8NativePathBuf;

use crate::epub::EpubFile;
use crate::state::AppState;
use crate::utils::get_config_dir_path;
use crate::zip::EntryNotFound;

type BytesResponse = Response<Cursor<Vec<u8>>>;

pub fn start_http_server(app: AppHandle) -> Result<u16> {
    let server = Server::http("127.0.0.1:0").map_err(|e| anyhow!("{e}"))?;
    let port = server.server_addr().to_ip().unwrap().port();

    let cpus = num_cpus::get().min(8);
    let pool = ThreadPoolBuilder::new().num_threads(cpus).build()?;

    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let app = app.clone();
            pool.spawn(move || {
                if let Err(e) = handle_request(app, request) {
                    eprintln!("Error handling request:\n{:?}", e);
                }
            });
        }
    });

    Ok(port)
}

fn handle_request(app: AppHandle, request: Request) -> Result<()> {
    if request.method() != &Method::Get {
        let response = Response::new_empty(StatusCode(405));
        return Ok(request.respond(response)?);
    }

    let uri = tauri::http::Uri::try_from(request.url())?;

    let response = match uri.path() {
        "/" => handle_root_request(),
        path if path.starts_with("/book/") => handle_book_request(app, path),
        path if path.starts_with("/cover/") => handle_thumbnail_request(app, path),
        path if path.starts_with("/static/") => handle_asset_request(app, &request, path),
        path => Ok(make_response(404, format!("Not Found: {path}"))),
    }?;

    Ok(request.respond(response)?)
}

fn handle_root_request() -> Result<BytesResponse> {
    let response = make_response(204, []);
    Ok(response)
}

fn handle_book_request(app: AppHandle, path: &str) -> Result<BytesResponse> {
    static PARAMS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^/book/([A-Za-z0-9_-]+)/(.+)$").unwrap());

    let params = PARAMS.captures(path).and_then(|captures| {
        let id = captures.get(1)?.as_str();
        let path = captures.get(2)?.as_str();
        Some((id, path))
    });

    let Some((id, path)) = params else {
        let response = make_response(404, format!("Not Found: {path}"));
        return Ok(response);
    };

    let state = app.state::<AppState>();
    let mut epubs = state.epubs().write();
    let epub = epubs.get_mut(id).context("Book not opened")?;

    let content = match epub.read_file(path) {
        Ok(content) => content,
        Err(e) => match e.root_cause().downcast_ref::<EntryNotFound>() {
            Some(EntryNotFound) => {
                let response = make_response(404, format!("File not found: {path}"));
                return Ok(response);
            }
            _ => {
                let response = make_response(500, format!("Failed to read file: {path}\n{e}"));
                return Ok(response);
            }
        },
    };

    let media_type = epub.get_media_type(path).unwrap_or("text/plain");
    let content_type = Header::from_str(&format!("Content-Type: {media_type}")).unwrap();
    let response = make_response(200, content).with_header(content_type);

    Ok(response)
}

fn handle_thumbnail_request(app: AppHandle, path: &str) -> Result<BytesResponse> {
    static PARAMS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^/cover/([A-Za-z0-9_-]+)\\.png$").unwrap());

    let params = PARAMS
        .captures(path)
        .and_then(|captures| captures.get(1))
        .map(|id| id.as_str());

    let Some(id) = params else {
        let response = make_response(404, format!("Not Found: {path}"));
        return Ok(response);
    };

    let thumbnail_dir = get_config_dir_path()?.join("cover");
    let thumbnail_path = thumbnail_dir.join(format!("{id}.png"));

    if let Ok(data) = std::fs::read(&thumbnail_path) {
        let content_type = Header::from_str("Content-Type: image/png").unwrap();
        let response = make_response(200, data).with_header(content_type);
        return Ok(response);
    }

    let state = app.state::<AppState>();
    let library = state.library().lock();
    let Some(book_path) = library.books().get(id).map(|book| book.path.clone()) else {
        let response = make_response(404, format!("Book Not Found in library: {id}"));
        return Ok(response);
    };
    drop(library);

    let Ok(mut epub) = EpubFile::open(Utf8NativePathBuf::from(&book_path)) else {
        let response = make_response(500, format!("Failed to open epub: {book_path}"));
        return Ok(response);
    };

    let thumbnail = match make_cover_thumbnail(&mut epub) {
        Ok(thumbnail) => thumbnail,
        Err(e) => {
            let response = make_response(500, format!("Failed to get book cover: {id}\n{e}"));
            return Ok(response);
        }
    };

    let _ = thumbnail.save(thumbnail_path);

    let mut data: Vec<u8> = Vec::new();
    thumbnail.write_to(&mut Cursor::new(&mut data), ImageOutputFormat::Png)?;

    let content_type = Header::from_str("Content-Type: image/png").unwrap();
    let response = make_response(200, data).with_header(content_type);

    Ok(response)
}

fn handle_asset_request(app: AppHandle, request: &Request, path: &str) -> Result<BytesResponse> {
    let built_at = build_time::build_time_utc!("%a, %d %b %Y %T GMT");
    for header in request.headers() {
        if header.field.equiv("If-Modified-Since") && header.value == built_at {
            return Ok(make_response(304, []));
        }
    }

    let resolver = app.asset_resolver();
    let Some(asset) = resolver.get(path.to_string()) else {
        let response = make_response(404, format!("Not Found: {path}"));
        return Ok(response);
    };

    let ctyp = format!("Content-Type: {}", asset.mime_type);
    let clen = format!("Content-Length: {}", asset.bytes.len());
    let lmod = format!("Last-Modified: {}", built_at);

    let response = make_response(200, asset.bytes)
        .with_header(Header::from_str(&ctyp).unwrap())
        .with_header(Header::from_str(&clen).unwrap())
        .with_header(Header::from_str(&lmod).unwrap())
        .with_header(Header::from_str("Cache-Control: max-age=31536000").unwrap());

    Ok(response)
}

fn make_response<T: Into<Vec<u8>>>(status: u16, body: T) -> BytesResponse {
    Response::from_data(body)
        .with_status_code(status)
        .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
}

fn make_cover_thumbnail(epub: &mut EpubFile) -> Result<DynamicImage> {
    use image::imageops::FilterType;
    use image::io::Reader;

    let path = epub.rootfile().get_cover_path().context("No cover")?;
    let data = epub.read_file(&path)?;

    let reader = Reader::new(Cursor::new(data)).with_guessed_format()?;
    let image = reader.decode()?;
    let thumbnail = image.resize_to_fill(240, 360, FilterType::Triangle);

    Ok(thumbnail)
}
