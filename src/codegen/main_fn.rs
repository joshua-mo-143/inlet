use proc_macro2::{Ident, Span, TokenStream};
use crate::codegen::axum_auth::{auth_router};
use quote::quote;
use syn::{File};

use crate::codegen::axum_snippets;
use crate::codegen::queries::QueryGen;
use crate::cli::Config;

pub fn main_function(cfg: Config) -> (File, String) {

    let (appstate, with_state, state_declare, dbmacro) = axum_snippets::state_snippets(cfg.clone());
    let (routers, crud_nest, mut useitems) = axum_snippets::axum_crud_routes(cfg.routes);

    let secretsmacro = if cfg.secrets {
        Some(quote! {#[shuttle_secrets::Secrets] secrets: SecretStore,})
    } else {
        None
    };

    let auth_router = if cfg.auth {
        Some(auth_router())
    } else {
        None
    };

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

pub fn axum_crud_routes(tablename: &str) -> File {
    let query_data = QueryGen::create_query_data(tablename);

    let mut v: Vec<char> = tablename.chars().collect();
    v[0] = v[0].to_uppercase().next().unwrap();
    let tablename: String = v.into_iter().collect();

    let structname = Ident::new(&tablename, Span::call_site());
    
    let query_fn_names: Vec<Ident> = query_data.iter().map(|x| x.query_fn_name.clone()).collect();
    let queries: Vec<String> = query_data.iter().map(|x| x.query.clone()).collect();
    let paths: Vec<Option<TokenStream>> = query_data.iter().map(|x| x.path.clone()).collect();
    let binds: Vec<Option<TokenStream>> = query_data.iter().map(|x| x.bind.clone()).collect();
    let fetch_mode: Vec<TokenStream> = query_data.iter().map(|x| x.fetch_mode.clone()).collect();
    let querytype: Vec<TokenStream> = query_data.iter().map(|x| x.querytype.clone()).collect();
    let response: Vec<TokenStream> = query_data.iter().map(|x| x.response.clone()).collect();
    let declarations: Vec<TokenStream> = query_data.iter().map(|x| x.declaration.clone()).collect();
    let error_handling: Vec<TokenStream> = query_data.iter().map(|x| x.error_handling.clone()).collect();

    let endpoint = axum_endpoint();

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

    syn::parse_file(&routes.to_string()).unwrap()
}
pub fn axum_endpoint() -> TokenStream {
    quote! {
        Result<impl IntoResponse, impl IntoResponse>
    }
}