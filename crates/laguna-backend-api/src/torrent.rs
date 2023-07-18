use std::fs::{self, File};

use actix_web::{get, patch, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use digest::Digest;
use laguna_backend_middleware::filters::torrent::{TorrentFilter, DEFAULT_TORRENT_FILTER_LIMIT};
use laguna_backend_model::torrent::{Torrent, TorrentDTO, TorrentPatchDTO, TorrentPutDTO};
use sha2::Sha256;
use sqlx::PgPool;
use std::io::Read;
use std::io::Write;
use uuid::Uuid;

use crate::{
    error::{APIError, TorrentError},
    state::TorrentState,
};

/// `GET /api/torrent/{id}`
/// # Example
/// ## Request
/// ```sh
/// curl -X GET \
///      -i 'http://127.0.0.1:6969/api/torrent/00f045ac-1f4d-4601-b2e3-87476dc462e6'
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4'
/// ```
/// ## Response
/// HTTP/1.1 200 OK
/// ```json
/// {
///    "title": "test",
///    "file_name": "test_upload",
///    "nfo": null,
///    "info_hash": "aae8b4b6a0b9b6b5b4b6b5b4b6b5b4b6b5b4b6b5",
///    "uploaded_at": "2023-07-10T12:42:32.396647Z",
///    "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///    "modded_by": null,
///    "payload": ""
/// }
/// ```
/// This only gets you torrent metadata (stored in DB).
#[get("/{id}")]
pub async fn get_torrent(
    id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    let torrent = sqlx::query_as::<_, Torrent>("SELECT * FROM \"Torrent\" WHERE id = $1")
        .bind(id.into_inner())
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| TorrentError::DoesNotExist)?;
    Ok(HttpResponse::Ok().json(TorrentDTO::from(torrent)))
}

/// `GET /api/torrent/{info_hash}`
/// # Example
/// ## Request
/// ```sh
/// curl -X GET \
///      -i 'http://127.0.0.1:6969/api/torrent/aae8b4b6a0b9b6b5b4b6b5b4b6b5b4b6b5b4b6b5' \
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4'
/// ```
/// ## Response
/// HTTP/1.1 200 OK
///
/// ```json
/// {
///   "title": "test",
///   "file_name": "test_upload",
///   "nfo": null,
///   "info_hash": "aae8b4b6a0b9b6b5b4b6b5b4b6b5b4b6b5b4b6b5",
///   "uploaded_at": "2023-07-10T12:42:32.396647Z",
///   "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///   "modded_by": null,
///   "payload": ""
/// }
/// This only gets you torrent metadata (stored in DB).
#[get("/{info_hash}")]
pub async fn get_torrent_with_info_hash(
    info_hash: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    let torrent = sqlx::query_as::<_, Torrent>("SELECT * FROM \"Torrent\" WHERE info_hash = $1")
        .bind(info_hash.into_inner())
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| TorrentError::DoesNotExist)?;
    Ok(HttpResponse::Ok().json(TorrentDTO::from(torrent)))
}

/// `GET /api/torrent/`
/// # Example
/// ## Request
/// ```sh
/// curl -X GET \
///      -i 'http://127.0.0.1:6969/api/torrent/' \
///      -H 'Content-Type: application/json' \
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4'
///      --data '{
///         "uploaded_at_min": "2023-07-10T12:42:32.396647Z",
///         "uploaded_at_max": null,
///         "uploaded_by": null,
///         "order_by": {
///            "TorrentOrderBy": {
///                "field": "UploadedAt",
///                "order": "Desc",
///            }
///         }
///      }'
/// ```
/// ## Response
/// HTTP/1.1 200 OK
/// ```json
/// [
///   {
///     "title": "test",
///     "file_name": "test_upload",
///     "nfo": null,
///     "info_hash": "aae8b4b6a0b9b6b5b4b6b5b4b6b5b4b6b5b4b6b5",
///     "uploaded_at": "2023-07-10T12:42:32.396647Z",
///     "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///     "modded_by": null,
///     "payload": ""
///   },
///   {
///     "title": "test2",
///     "file_name": "test_upload2",
///     "nfo": null,
///     "info_hash": "aae8b4b6a0b9b6b5b4b6b5b4b6b5b4b6b5b4b6b5",
///     "uploaded_at": "2023-07-10T12:42:31.396647Z",
///     "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///     "modded_by": null,
///     "payload": ""
///   }
/// ]
/// ```
#[get("/")]
pub async fn get_torrents_with_filter(
    filter: web::Json<TorrentFilter>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    // Dynamic query generation is still being worked on: https://github.com/launchbadge/sqlx/issues/291
    // See: https://github.com/launchbadge/sqlx/issues/364
    let torrents = sqlx::query_as::<_, Torrent>(&format!(
        r#"
        SELECT * 
        FROM "Torrent" 
        INNER JOIN "User" USING (id)
        WHERE 
        (uploaded_at >= $1 AND uploaded_at <= $2) AND
        (($3 IS NULL and username IS NULL) OR username = $3)
        {order_by}
        LIMIT $4
        "#,
        order_by = match filter.order_by {
            None => String::from(""),
            Some(ref order_by) => order_by.to_string(),
        }
    ))
    .bind(
        filter
            .uploaded_at_min
            .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC),
    )
    .bind(
        filter
            .uploaded_at_max
            .unwrap_or_else(|| DateTime::<Utc>::MAX_UTC),
    )
    .bind(&filter.uploaded_by)
    .bind(filter.limit.unwrap_or_else(|| DEFAULT_TORRENT_FILTER_LIMIT))
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(
        torrents
            .into_iter()
            .map(|torrent| TorrentDTO::from(torrent))
            .collect::<Vec<TorrentDTO>>(),
    ))
}

/// `PATCH /api/torrent/`
/// # Example
/// ## Request
/// ```sh
/// curl -X PATCH \
///      -i 'http://127.0.0.1:6969/api/torrent/' \
///      -H 'Content-Type: application/json' \
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4' \
///      --data '{
///         "title": "TEST (2020)",
///         "file_name": "test_upload",
///         "nfo": null,
///         "modded_by": null
///      }'
/// Updates torrent metadata (not file).
/// Certain fields are not allowed to be updated.
/// Returns updated [`TorrentDTO`].
#[patch("/")]
pub async fn patch_torrent(
    torrent_dto: web::Json<TorrentPatchDTO>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    Ok(HttpResponse::Ok().json(TorrentDTO::from(
        sqlx::query_as::<_, Torrent>(
            r#"
    UPDATE "Torrent" 
    SET file_name = $1, nfo = $2, modded_by = $3
    WHERE title = $4
    RETURNING *
    "#,
        )
        .bind(&torrent_dto.file_name)
        .bind(&torrent_dto.nfo)
        .bind(&torrent_dto.modded_by)
        .bind(&torrent_dto.title)
        .fetch_one(pool.get_ref())
        .await?,
    )))
}

/// `GET /api/torrent/download/{id}`
/// # Example
/// ## Request
/// ```sh
/// curl -X GET \
///      -i 'http://127.0.0.1:6969/api/torrent/download/00f045ac-1f4d-4601-b2e3-87476dc462e6'
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4'
/// ```
/// ## Response
/// HTTP/1.1 200 OK
/// ```json
/// {
///     "id": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///     "title": "test",
///     "file_name": "test_upload",
///     "nfo": null,
///     "info_hash": "e6c4b2e3b1f4d4601b2e3b1f4d4601b2e3b1f4d4601b2e3b1f4d4601b2e3b1f4",
///     "uploaded_at": "2021-07-04T10:18:17.391698Z",
///     "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///     "modded_by": null,
///     "payload": ""
/// }
/// ```
/// Actually downloads .torrent with file name {id}.torrent.
/// But when downloading renames it to {file_name}.torrent.
/// We store files in `torrents/` folder on localfs in {id}.torrent format to ensure uniqueness.
/// TODO: Use magnet links.
#[get("/{id}")]
pub async fn get_torrent_download(
    id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    let torrent = sqlx::query_as::<_, Torrent>("SELECT * FROM \"Torrent\" WHERE id = $1")
        .bind(id.into_inner())
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| TorrentError::DoesNotExist)?;
    let mut file = File::open(format!("torrents/{}.torrent", torrent.id))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut torrent_dto = TorrentDTO::from(torrent);
    torrent_dto.payload = bytes;
    Ok(HttpResponse::Ok().json(torrent_dto))
}

/// `PUT /api/torrent/upload`
/// # Example
/// ## Request
/// ```sh
/// curl -X PUT \
///      -i 'http://127.0.0.1:6969/api/torrent/upload' \
///      -H 'X-Access-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg5OTMwNTksImlhdCI6MTY4ODk5Mjk5OSwiaWQiOiIwMGYwNDVhYy0xZjRkLTQ2MDEtYjJlMy04NzQ3NmRjNDYyZTYiLCJ1c2VybmFtZSI6InRlc3QiLCJmaXJzdF9sb2dpbiI6IjIwMjMtMDctMTBUMTI6NDI6MzIuMzk2NjQ3WiIsImxhc3RfbG9naW4iOiIyMDIzLTA3LTEwVDEyOjQzOjE5LjIxNjA0N1oiLCJhdmF0YXJfdXJsIjpudWxsLCJyb2xlIjoiTm9ybWllIiwiYmVoYXZpb3VyIjoiTHVya2VyIiwiaXNfYWN0aXZlIjp0cnVlLCJoYXNfdmVyaWZpZWRfZW1haWwiOmZhbHNlLCJpc19oaXN0b3J5X3ByaXZhdGUiOnRydWUsImlzX3Byb2ZpbGVfcHJpdmF0ZSI6dHJ1ZX0.izClLn6kANl2kpIv2QqzmKJy7tmpNZqKKvcd4RoGW_c' \
///      -H 'X-Refresh-Token: eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2ODg0NjkzMzksImlhdCI6MTY4ODQ2NzUzOSwidXNlcm5hbWUiOiJ0ZXN0IiwiZW1haWwiOiJ0ZXN0QGxhZ3VuYS5pbyIsInBhc3N3b3JkIjoiZWNkNzE4NzBkMTk2MzMxNmE5N2UzYWMzNDA4Yzk4MzVhZDhjZjBmM2MxYmM3MDM1MjdjMzAyNjU1MzRmNzVhZSIsImZpcnN0X2xvZ2luIjoiMjAyMy0wNy0wNFQxMDoxODoxNy4zOTE2OThaIiwibGFzdF9sb2dpbiI6IjIwMjMtMDctMDRUMTA6MTg6MTcuMzkxNjk4WiIsImF2YXRhcl91cmwiOm51bGwsInJvbGUiOiJOb3JtaWUiLCJpc19hY3RpdmUiOnRydWUsImhhc192ZXJpZmllZF9lbWFpbCI6ZmFsc2UsImlzX2hpc3RvcnlfcHJpdmF0ZSI6dHJ1ZSwiaXNfcHJvZmlsZV9wcml2YXRlIjp0cnVlfQ.5fdMnIj0WqV0lszANlJD_x5-Oyq2h8bhqDkllz1CGg4' \
///      -H 'Content-Type: application/json' \
///     --data '{
///        "title": "test",
///        "file_name": "test_upload",
///        "nfo": null,
///        "uploaded_by": "00f045ac-1f4d-4601-b2e3-87476dc462e6",
///        "modded_by": null,
///        "payload": ""
///     }'
/// ```
/// ## Response
/// HTTP/1.1 200 OK
///
/// ```json
/// "UploadSuccess"
/// ```
/// TODO: Right now, we send it via Json body, but we should use multipart/form-data.
#[put("/")]
pub async fn put_torrent(
    torrent_dto: web::Json<TorrentPutDTO>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, APIError> {
    let torrent = sqlx::query_as::<_, Torrent>("SELECT * FROM \"Torrent\" WHERE title = $1")
        .bind(&torrent_dto.title)
        .fetch_optional(pool.get_ref())
        .await?;

    match torrent {
        Some(_) => Ok(HttpResponse::AlreadyReported().json(TorrentState::AlreadyExists)),
        None => {
            let mut transaction = pool.begin().await?;
            let torrent = sqlx::query_as::<_, Torrent>(
                r#"
            INSERT INTO "Torrent" (title, file_name, nfo, info_hash, uploaded_at, uploaded_by) 
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
            )
            .bind(&torrent_dto.title)
            .bind(&torrent_dto.file_name)
            .bind(&torrent_dto.nfo)
            .bind(format!("{:x}", Sha256::digest(&torrent_dto.payload))) // TODO: Hash only info section of Torrent. This is fine for now, but redundant.
            .bind(Utc::now())
            .bind(&torrent_dto.uploaded_by)
            .fetch_one(&mut transaction)
            .await?;

            // Transaction is rolled back if file creation fails.
            fs::create_dir_all("/torrents")?;
            let mut file = File::create(format!("/torrents/{}.torrent", torrent.id))?;
            file.write_all(&torrent_dto.payload)?;

            // Transaction is rolled back if file create time cannot be fetched.
            let file_create_time = file.metadata().map(|metadata| metadata.created())??;

            sqlx::query("UPDATE \"Torrent\" SET uploaded_at = $1 WHERE id = $2")
                .bind(DateTime::<Utc>::from(file_create_time))
                .bind(torrent.id)
                .execute(&mut transaction)
                .await?;

            transaction.commit().await?;
            Ok(HttpResponse::Ok().json(TorrentState::UploadSuccess))
        }
    }
}