use proc_macro2::{Span, Ident, TokenStream};
use crate::cli::Config;
use quote::quote;
use indoc::{indoc, formatdoc};

type Snippets = (Option<TokenStream>, TokenStream, Option<TokenStream>, Option<TokenStream>);

pub fn state_snippets(cfg: Config) -> Snippets {

    let key = if cfg.auth {Some(quote! {pub key: Key,})} else {None};
    let keygen = if cfg.auth {Some(quote! {key: Key::generate()})} else {None};
    let keyimpl = if cfg.auth {Some(quote! {
        impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
    })} else {None};
	
    let appstate = if cfg.auth | cfg.crud {
                Some(quote! {
            #[derive(Clone)]
            pub struct AppState {
                pub db: PgPool,
                #key
            }
            #keyimpl
        })
    } else {
        None
    };

    let with_state = if cfg.auth | cfg.crud {
        quote! {
            .with_state(state);
        }
    } else {
        quote! {
            ;
        }
    };

    let state_declare = if cfg.auth | cfg.crud {
        Some(quote! {
            let state = AppState { db, #keygen };
        })
    } else { None };
    
    let db_macro = if cfg.auth | cfg.crud {
        Some(quote! {
            #[shuttle_shared_db::Postgres] db: PgPool,
        })
    } else { None };

	(appstate, with_state, state_declare, db_macro)
}

pub fn axum_crud_routes(routes: Option<Vec<String>>) -> (TokenStream, TokenStream, String) {
    
    let mut routers: Vec<TokenStream> = Vec::new();
    let mut nest: Vec<TokenStream> = Vec::new();
    let mut useitems: String = String::new();

if let Some(routes) = routes {
        for route in routes {
        let router_name = Ident::new(&format!("{route}_router"), Span::call_site());
        let get_all_route = Ident::new(&format!("get_all_{route}"), Span::call_site());
        let get_one_route = Ident::new(&format!("get_{route}_by_id"), Span::call_site());
        let create_route = Ident::new(&format!("create_{route}"), Span::call_site());
        let update_route = Ident::new(&format!("update_{route}_by_id"), Span::call_site());         
        let delete_route = Ident::new(&format!("delete_{route}_by_id"), Span::call_site());
        let route_location = format!("/{route}");         
        routers.push(
                quote! {
                    let #router_name = Router::new()
                        .route("/", get(#get_all_route).post(#create_route))
                        .route("/:id", get(#get_one_route).patch(#update_route).delete(#delete_route));
                }
            );
        nest.push(
                quote! {
                    .nest(#route_location, #router_name)
                }
            );

        let leftbrace = indoc! {"{"};
        let rightbrace = indoc! {"}"};
            useitems.push_str(&formatdoc! {"use crate::routes::{route}::{leftbrace}{get_all_route}, {get_one_route}, {create_route}, {update_route}, {delete_route}{rightbrace};\n"});
        }
    }

    
    let crud_routers = quote! {
        #(
            #routers
        )*
    };

    let nesting = quote! {
        #(
            #nest
        )*
    };

    (crud_routers, nesting, useitems)
}  