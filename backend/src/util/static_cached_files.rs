//! Custom handler and options for static file serving _with cache control_.
//!
//! Mostly stolen from [`StaticFiles`](rocket_contrib::serve::StaticFiles).

use log::error;
use rocket::handler::{Handler, Outcome};
use rocket::http::{
    hyper::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH},
    uri::Segments,
    Header, Method, Status,
};
use rocket::response::{NamedFile, Responder, Response};
use rocket::{Data, Request, Route};
use std::io;
use std::path::{Path, PathBuf};

/// Custom handler for serving static files with Cache Control
///
/// It is based on the StaticFiles handler from rocket.
/// The difference is that it generates an etag for each file based on its path, and
/// the start time of the program. Meaning that files **must not be changed** after the
/// program has started.
#[derive(Clone)]
pub struct StaticCachedFiles {
    root: PathBuf,
    rank: isize,
    max_age: u32,
}

impl StaticCachedFiles {
    /// The default rank use by `StaticCachedFiles` routes.
    const DEFAULT_RANK: isize = 10;

    /// The default `max-age` cache-control directive
    const DEFAULT_MAX_AGE: u32 = 0;

    /// Constructs a new `StaticCachedFiles` that serves files from the file system `path`.
    pub fn from<P: AsRef<Path>>(path: P) -> Self {
        StaticCachedFiles::new(path)
    }

    /// Constructs a new `StaticCachedFiles` that serves files from the file system `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        use rocket::yansi::Paint;

        let path = path.as_ref();
        if !path.is_dir() {
            error!("`StaticCachedFiles` supplied with invalid path");
            error!("'{}' is not a directory", Paint::white(path.display()));
            panic!("refusing to continue due to invalid static files path");
        }

        StaticCachedFiles {
            root: path.into(),
            max_age: Self::DEFAULT_MAX_AGE,
            rank: Self::DEFAULT_RANK,
        }
    }

    /// Sets the rank for generated routes to `rank`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # extern crate rocket_contrib;
    /// use rocket_contrib::serve::{StaticCachedFiles, Options};
    ///
    /// // A `StaticCachedFiles` created with `from()` with routes of rank `3`.
    /// StaticCachedFiles::from("/public").rank(3);
    ///
    /// // A `StaticCachedFiles` created with `new()` with routes of rank `-15`.
    /// StaticCachedFiles::new("/public", Options::Index).rank(-15);
    /// ```
    pub fn rank(mut self, rank: isize) -> Self {
        self.rank = rank;
        self
    }

    /// Sets the rank for generated routes to `rank`.
    pub fn max_age(mut self, max_age: u32) -> Self {
        self.max_age = max_age;
        self
    }
}

impl Into<Vec<Route>> for StaticCachedFiles {
    fn into(self) -> Vec<Route> {
        let non_index = Route::ranked(self.rank, Method::Get, "/<path..>", self.clone());
        vec![non_index]
    }
}

fn generate_etag<P: AsRef<Path>>(path: P) -> String {
    use chrono::{DateTime, Utc};
    use lazy_static::lazy_static;
    use sha2::{Digest, Sha256};
    use std::os::unix::ffi::OsStrExt;

    lazy_static! {
        static ref START_TIME: DateTime<Utc> = Utc::now();
    }

    let mut hasher = Sha256::new();
    hasher.update(path.as_ref().as_os_str().as_bytes());
    hasher.update((*START_TIME).timestamp().to_be_bytes());

    let hash = hasher.finalize();
    hex::encode(hash)
}

struct CachedFile {
    etag: String,
    cache_control: String,
    file: Option<NamedFile>,
}

impl CachedFile {
    pub async fn open<P: AsRef<Path>>(
        path: P,
        req_etag: Option<String>,
        cache_control: String,
    ) -> io::Result<CachedFile> {
        let etag = generate_etag(&path);
        let file = match req_etag {
            Some(req_etag) if req_etag == etag => None,
            _ => Some(NamedFile::open(path).await?),
        };

        Ok(CachedFile {
            etag,
            cache_control,
            file,
        })
    }
}

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request) -> Result<Response<'static>, Status> {
        let mut response = match self.file {
            None => {
                let mut response = Response::new();
                response.set_status(Status::NotModified);
                response
            }
            Some(file) => file.respond_to(req)?,
        };

        response.adjoin_header(Header::new(ETAG.as_str(), self.etag));
        response.adjoin_header(Header::new(CACHE_CONTROL.as_str(), self.cache_control));

        Ok(response)
    }
}

#[rocket::async_trait]
impl Handler for StaticCachedFiles {
    async fn handle<'r, 's: 'r>(&'s self, req: &'r Request<'_>, data: Data) -> Outcome<'r> {
        // If this is not the route with segments, handle it only if the user
        // requested a handling of index files.
        let current_route = req.route().expect("route while handling");
        let is_segments_route = current_route.uri.path().ends_with(">");
        if !is_segments_route {
            return Outcome::forward(data);
        }

        // Otherwise, we're handling segments. Get the segments as a `PathBuf`,
        // only allowing dotfiles if the user allowed it.
        let allow_dotfiles = false;
        let path = req
            .get_segments::<Segments<'_>>(0)
            .and_then(|res| res.ok())
            .and_then(|segments| segments.into_path_buf(allow_dotfiles).ok())
            .map(|path| self.root.join(path));

        match path {
            None => Outcome::forward(data),
            Some(p) if p.is_dir() => Outcome::forward(data),
            Some(p) => {
                let req_etag = req
                    .headers()
                    .get_one(IF_NONE_MATCH.as_str())
                    .map(|etag| etag.to_string());
                let cache_control = format!("must-revalidate, max-age={}", self.max_age);
                if let Ok(Ok(response)) = CachedFile::open(p, req_etag, cache_control)
                    .await
                    .map(|file| file.respond_to(req))
                {
                    Outcome::Success(response)
                } else {
                    Outcome::Forward(data)
                }
            }
        }
    }
}
