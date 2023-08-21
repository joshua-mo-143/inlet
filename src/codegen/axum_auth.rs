use crate::codegen::main_fn::axum_endpoint;
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

fn register_route() -> TokenStream {
    let endpoint = axum_endpoint();

    quote! {
        pub async fn register(
            State(state): State<AppState>,
            Json(user): Json<LoginDetails>
        ) -> #endpoint {
            let hashed_password = hash(user.password, 10u32).unwrap();

            if let Err(e) = sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
                .bind(user.username)
                .bind(hashed_password)
                .execute(&state.db)
                .await {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error while registering: {e}")));
            }

            Ok(StatusCode::CREATED)
        }
    }
}

pub fn auth_router() -> TokenStream {
    quote! {
    let auth_router = Router::new()
        .route("/login", post(login))
        .route("/register", post(register));
    }
}

fn login_route() -> TokenStream {
    let endpoint = axum_endpoint();

    quote! {
        pub async fn login(
            State(state): State<AppState>,
            jar: PrivateCookieJar,
            Json(user): Json<LoginDetails>
        ) -> #endpoint {
            let res = match sqlx::query_as::<_, LoginDetails>("SELECT USERNAME, PASSWORD FROM users WHERE username = $1")
                .bind(user.username.clone())
                .fetch_one(&state.db)
                .await {
                Ok(res) => res,
                Err(_) => return Err((StatusCode::BAD_REQUEST, "Incorrect credentials".to_string()))
            };

            match verify(user.password, &res.password) {
                Ok(true) => {},
                Ok(false) => {return Err((StatusCode::BAD_REQUEST, "Incorrect credentials".to_string()))},
                Err(e) => {return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong trying to verify your password: {e}")))}
            }

            let session_id = "hello world!";

            let expires_at = Utc::now().naive_local() + ChronoDuration::seconds(3600);

            if let Err(e) = sqlx::query("INSERT INTO UserSessions
                (user_id, session_id, expires_at)
                VALUES
                ((SELECT ID FROM users WHERE username = $1)
                , $2, $3)
                ON CONFLICT (user_id)
                DO UPDATE SET 
                session_id = excluded.session_id,
                expires_at = excluded.expires_at")
                .bind(user.username)
                .bind(session_id)
                .bind(expires_at)
                .execute(&state.db)
                .await {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Something went wrong trying to give you a session: {e}")
                ));
            }

            let cookie = Cookie::build("foo", session_id)
                .domain(".app.localhost")
                .secure(true)
                .http_only(true)
                .max_age(TimeDuration::seconds(3600))
                .finish();

            Ok((
                jar.add(cookie),
                StatusCode::OK
            ))
        }
    }
}

pub fn auth_routes() -> File {
    let register = register_route();
    let login = login_route();

    let code = quote! {

            use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
    use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
    use bcrypt::{verify, hash};
    use serde::Deserialize;
    use chrono::{Duration as ChronoDuration, Utc};
    use time::Duration as TimeDuration;
    use crate::AppState;

            #[derive(Deserialize, sqlx::FromRow)]
            pub struct LoginDetails {
                pub username: String,
                pub password: String
            }

            #register

            #login
        };

    syn::parse_file(&code.to_string()).unwrap()
}

pub fn auth_middleware() -> File {
    let code = quote! {
        use serde::{Deserialize, Serialize};
        use crate::AppState;
        use axum_extra::extract::cookie::PrivateCookieJar;
        use axum::{http::{Request, StatusCode}, middleware::Next, response::IntoResponse, extract::State};

        #[derive(Clone, Deserialize, Serialize, sqlx::FromRow)]
        pub struct UserInfo {
            user_id: i32,
        }

        pub async fn check_authed_cookies<B>(
            State(state): State<AppState>,
            jar: PrivateCookieJar,
            mut req: Request<B>,
            next: Next<B>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            let Some(cookie) = jar.get("session").map(|cookie| cookie.value().to_owned()) else {
                return Err((StatusCode::FORBIDDEN, "Forbidden!".to_string()))
            };

            let user_info = match sqlx::query_as::<_, UserInfo>("SELECT user_id FROM usersessions WHERE session_id = $1")
                .bind(cookie)
                .fetch_one(&state.db)
                .await {
                Ok(res) => res,
                Err(_) => {return Err((StatusCode::FORBIDDEN, "Forbidden!".to_string()))}
            };

            req.extensions_mut().insert(user_info);
            Ok(next.run(req).await)
        }
    };

    syn::parse_file(&code.to_string()).unwrap()
}
