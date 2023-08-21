use crate::codegen::axum_auth::auth_router;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::File;

use crate::cli::{Config, Route};
use crate::codegen::axum_snippets;
use crate::codegen::queries::QueryGen;

pub fn main_function(cfg: Config, routes: Vec<Route>) -> (File, String) {
    let (appstate, with_state, state_declare, dbmacro) = axum_snippets::state_snippets(cfg.clone());
    let (routers, crud_nest, mut useitems) = axum_snippets::axum_crud_routes(routes.clone());

    let secretsmacro = if cfg.secrets {
        Some(quote! {#[shuttle_secrets::Secrets] secrets: SecretStore,})
    } else {
        None
    };

    if routes.iter().any(|x| x.auth_required == true) {
        useitems.push_str("use axum::middleware::from_fn_with_state;\n");
        useitems.push_str("use crate::middleware::auth::check_authed_cookies;\n");
        
    }

    let auth_router = if cfg.auth { Some(auth_router()) } else { None };

    let auth_nest = if cfg.auth {
        Some(quote! {.nest("/auth", auth_router)})
    } else {
        None
    };

    if cfg.crud | cfg.auth {
        useitems.push_str("use sqlx::PgPool;\n");
        useitems.push_str("use axum::routing::post;\n");
    }

    if cfg.auth {
        useitems.push_str("use crate::routes::auth::{login, register};\n");
        useitems.push_str("use axum_extra::extract::cookie::Key;\n");
        useitems.push_str("use axum::extract::FromRef;\n");
        useitems.push_str("mod middleware;\n");
    }

    if cfg.secrets {
        useitems.push_str("use shuttle_secrets::SecretStore;\n");
    }

    let main = quote! {
        use axum::{routing::get, Router};
        mod routes;

        #appstate

        #[shuttle_runtime::main]
        pub async fn main(
            #dbmacro
            #secretsmacro
        ) -> shuttle_axum::ShuttleAxum {
            #state_declare

            #routers
            #auth_router

            let router = Router::new()
                #crud_nest
                #auth_nest
                .route("/", get(hello_world))#with_state

            Ok(router.into())
        }


        pub async fn hello_world() -> &'static str {
            "Hello world!"
        }
    };

    let file = syn::parse_file(&main.clone().to_string()).unwrap();

    (file, useitems)
}

pub fn axum_crud_fns(route: Route, requires_auth: bool) -> (File, String) {
    let query_data = QueryGen::create_query_data(&route.name);

    let mut v: Vec<char> = route.name.chars().collect();
    v[0] = v[0].to_uppercase().next().unwrap();
    let tablename: String = v.into_iter().collect();

    let structname = Ident::new(&tablename, Span::call_site());

    let mut extra_deps = String::new();

    if requires_auth {
        extra_deps.push_str("use axum::Extension;\n");
        extra_deps.push_str("use crate::middleware::auth::UserInfo;\n")
    }

    let query_fn_names: Vec<Ident> = query_data.iter().map(|x| x.query_fn_name.clone()).collect();
    let queries: Vec<String> = query_data.iter().map(|x| x.query.clone()).collect();
    let paths: Vec<Option<TokenStream>> = query_data.iter().map(|x| x.path.clone()).collect();
    let binds: Vec<Option<TokenStream>> = query_data.iter().map(|x| x.bind.clone()).collect();
    let fetch_mode: Vec<TokenStream> = query_data.iter().map(|x| x.fetch_mode.clone()).collect();
    let querytype: Vec<TokenStream> = query_data.iter().map(|x| x.querytype.clone()).collect();
    let response: Vec<TokenStream> = query_data.iter().map(|x| x.response.clone()).collect();
    let declarations: Vec<TokenStream> = query_data.iter().map(|x| x.declaration.clone()).collect();
    let error_handling: Vec<TokenStream> = query_data
        .iter()
        .map(|x| x.error_handling.clone())
        .collect();

    let endpoint = axum_endpoint();

    let userinfo_ext = if requires_auth {
        Some(quote! {Extension(_userinfo): Extension<UserInfo>,})
    } else {
        None
    };

    let routes = quote! {
        use crate::AppState;
        use axum::{response::IntoResponse, http::StatusCode, extract::{Path, State}, Json};
        use chrono::{DateTime, Utc};
        use serde::{Serialize};

        #[derive(Serialize, sqlx::FromRow)]
        pub struct #structname {
            id: i32,
            created_at: DateTime<Utc>,
            last_updated: DateTime<Utc>
        }

        #(
            pub async fn #query_fn_names(
            State(state): State<AppState>,
            #userinfo_ext
            #paths
        ) -> #endpoint {

            #declarations #querytype(#queries)
                    #binds
                    #fetch_mode
                    .await {
                    #error_handling
            };

                #response
        }
        )*
    };

    let code = syn::parse_file(&routes.to_string()).unwrap();

    (code, extra_deps)
}
pub fn axum_endpoint() -> TokenStream {
    quote! {
        Result<impl IntoResponse, impl IntoResponse>
    }
}
