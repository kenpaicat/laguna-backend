use actix_http::{
  error::PayloadError,
  header::{self, HeaderMap, HeaderValue},
};
use actix_multipart::Multipart;
use actix_web::{test::TestRequest, web::Bytes};
use futures::stream;
use sqlx::PgPool;

mod common;

#[actix_web::test]
#[ignore = "Not yet implemented fully"]
#[cfg(not(tarpaulin))]
async fn test_get_torrent() {}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "Not yet implemented fully"]
#[cfg(not(tarpaulin))]
async fn test_put_torrent(pool: PgPool) -> sqlx::Result<()> {
  let app = common::setup_test(&pool).await;
  let (_, _user_dto, access_token, refresh_token) = common::new_user(&app).await;
  let mut hmp = HeaderMap::new();
  // hmp.append(HeaderName::from_static("X-Access-Token"), access_token);
  // hmp.append(HeaderName::from_static("X-Refresh-Token"), refresh_token);
  hmp.append(
    header::CONTENT_TYPE,
    HeaderValue::from_static("multipart/form-data"),
  );
  hmp.append(
    header::CONTENT_DISPOSITION,
    HeaderValue::from_static("form-data; filename=\"alice.torrent\""),
  );
  let bytes = include_bytes!("fixtures/webtorrent-fixtures/fixtures/alice.torrent");
  let new_bytes = bytes.map(|b| Result::<Bytes, PayloadError>::Ok(Bytes::from_iter(vec![b])));
  let _multipart = Multipart::new(&hmp, stream::iter(new_bytes));
  let _res = common::as_logged_in(
    access_token,
    refresh_token,
    TestRequest::put().uri("/api/torrent"),
    &app,
  )
  .await
  .unwrap();
  Ok(())
}

#[actix_web::test]
#[ignore = "Not yet implemented fully"]
#[cfg(not(tarpaulin))]
async fn test_patch_torrent() {}
