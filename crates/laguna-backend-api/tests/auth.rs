use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp, Authority, TokenSigner};
use actix_web::{
    dev::{Service, ServiceResponse},
    http::{header, StatusCode},
    test::{init_service, TestRequest},
    web, App,
};

use chrono::{Duration, Utc};
use cookie::Cookie;
use env_logger;

use jwt_compact::{
    alg::{Hs256, Hs256Key},
    TimeOptions,
};
use laguna_backend_api::{login::login, register::register, user::me};
use laguna_backend_model::{
    login::LoginDTO,
    user::{Behaviour, Role, UserDTO},
};

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Once};

// Initialize env_logger only once.
static ENV_LOGGER_SETUP: Once = Once::new();

async fn setup() -> PgPool {
    ENV_LOGGER_SETUP.call_once(|| {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL not set"))
        .await
        .expect("Unable to connect to test database");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Couldn't run migrations");

    pool
}

async fn teardown(pool: PgPool) {
    sqlx::query("DELETE FROM \"User\"")
        .execute(&pool)
        .await
        .expect("Failed to cleanup \"User\" table");
    pool.close().await;
}

#[actix_web::test]
async fn test_register() {
    let pool = setup().await;
    let app = init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api/user/auth").service(register)),
    )
    .await;
    let req = TestRequest::post()
        .set_json(UserDTO {
            username: String::from("test"),
            email: String::from("test@laguna.io"),
            password: String::from("test123"),
            avatar_url: None,
            role: Role::Admin,
            behaviour: Behaviour::Lurker,
            is_active: None,
            is_history_private: None,
            first_login: None,
            last_login: None,
            has_verified_email: None,
            is_profile_private: None,
        })
        .uri("/api/user/auth/register");
    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Test already exists
    let req = TestRequest::post()
        .set_json(UserDTO {
            username: String::from("test"),
            email: String::from("test@laguna.io"),
            password: String::from("test123"),
            avatar_url: None,
            role: Role::Admin,
            behaviour: Behaviour::Lurker,
            is_active: None,
            is_history_private: None,
            first_login: None,
            last_login: None,
            has_verified_email: None,
            is_profile_private: None,
        })
        .uri("/api/user/auth/register");

    let res = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::ALREADY_REPORTED);

    teardown(pool).await;
}

#[actix_web::test]
async fn test_login() {
    let pool = setup().await;
    let key = Hs256Key::new("some random test shit");
    let authority = Authority::<UserDTO, Hs256, _, _>::new()
        .refresh_authorizer(|| async move { Ok(()) })
        .token_signer(Some(
            TokenSigner::new()
                .signing_key(key.clone())
                .algorithm(Hs256)
                .build()
                .expect("Cannot create token signer"),
        ))
        .verifying_key(key.clone())
        .build()
        .expect("Cannot create key authority");
    let app = init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/user/auth")
                    .service(register)
                    .service(login),
            )
            .use_jwt(authority, web::scope("/api")),
    )
    .await;

    let req = TestRequest::post()
        .set_json(UserDTO {
            username: String::from("test_login"),
            email: String::from("test_login@laguna.io"),
            password: String::from("test123"),
            avatar_url: None,
            role: Role::Admin,
            behaviour: Behaviour::Lurker,
            is_active: None,
            is_history_private: None,
            first_login: None,
            last_login: None,
            has_verified_email: None,
            is_profile_private: None,
        })
        .uri("/api/user/auth/register");
    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Test login with username
    let req = TestRequest::post()
        .set_json(LoginDTO {
            username_or_email: String::from("test_login"),
            password: String::from("test123"),
            login_timestamp: Utc::now(),
        })
        .uri("/api/user/auth/login");

    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(header::SET_COOKIE));

    // Test login with email
    let req = TestRequest::post()
        .set_json(LoginDTO {
            username_or_email: String::from("test_login@laguna.io"),
            password: String::from("test123"),
            login_timestamp: Utc::now(),
        })
        .uri("/api/user/auth/login");

    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(header::SET_COOKIE));

    // Test login with wrong password
    let req = TestRequest::post()
        .set_json(LoginDTO {
            username_or_email: String::from("test_login"),
            password: String::from("seiufhoifhjqow"),
            login_timestamp: Utc::now(),
        })
        .uri("/api/user/auth/login");

    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Test login with wrong username
    let req = TestRequest::post()
        .set_json(LoginDTO {
            username_or_email: String::from("test_loginx"),
            password: String::from("test123"),
            login_timestamp: Utc::now(),
        })
        .uri("/api/user/auth/login");

    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    teardown(pool).await;
}

#[actix_web::test]
async fn test_access_and_refresh_token() {
    let pool = setup().await;
    let key = Hs256Key::new("some random test shit");
    let authority = Authority::<UserDTO, Hs256, _, _>::new()
        .refresh_authorizer(|| async move { Ok(()) })
        // .enable_header_tokens(true) // see comment below
        .token_signer(Some(
            TokenSigner::new()
                .signing_key(key.clone())
                .algorithm(Hs256)
                .time_options(TimeOptions::from_leeway(Duration::nanoseconds(5))) // to make sure refresh is triggered. TODO: this is kind of best-effort like, can we explicitly test this?
                .build()
                .expect("Cannot create token signer"),
        ))
        .verifying_key(key.clone())
        .build()
        .expect("Cannot create key authority");

    let app = init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/user/auth")
                    .service(register)
                    .service(login),
            )
            .use_jwt(
                authority,
                web::scope("/api").service(web::scope("/user").service(me)),
            ),
    )
    .await;

    let req = TestRequest::post()
        .set_json(UserDTO {
            username: String::from("test_access_refresh"),
            email: String::from("test_access_refresh@laguna.io"),
            password: String::from("test123"),
            avatar_url: None,
            role: Role::Admin,
            behaviour: Behaviour::Lurker,
            is_active: None,
            is_history_private: None,
            first_login: None,
            last_login: None,
            has_verified_email: None,
            is_profile_private: None,
        })
        .uri("/api/user/auth/register");
    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let req = TestRequest::post()
        .set_json(LoginDTO {
            username_or_email: String::from("test_access_refresh"),
            password: String::from("test123"),
            login_timestamp: Utc::now(),
        })
        .uri("/api/user/auth/login");

    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    let mut cookies = res.headers().get_all(header::SET_COOKIE);
    let access_token = cookies.next().unwrap().to_str().unwrap();
    let refresh_token = cookies.next().unwrap().to_str().unwrap();
    assert_eq!(cookies.next(), None);

    let req = TestRequest::get()
        .uri("/api/user/me")
        .cookie(Cookie::parse(access_token).unwrap())
        .cookie(Cookie::parse(refresh_token).unwrap());
    let res: ServiceResponse = app.call(req.to_request()).await.unwrap();
    assert_eq!(res.status(), 200);

    teardown(pool).await;
}
