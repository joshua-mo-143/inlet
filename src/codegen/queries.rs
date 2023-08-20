use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

#[derive(Clone)]
pub struct QueryGen {
    pub query_fn_name: Ident,
    pub query: String,
    pub path: Option<TokenStream>,
    pub bind: Option<TokenStream>,
    pub fetch_mode: TokenStream,
    pub querytype: TokenStream,
    pub response: TokenStream,
    pub declaration: TokenStream,
    pub error_handling: TokenStream,
}

impl QueryGen {
    fn get_all(tablename: &str) -> Self {

    let mut v: Vec<char> = tablename.chars().collect();
    v[0] = v[0].to_uppercase().next().unwrap();
    let tablename_titlecase: String = v.into_iter().collect();

    let structname = Ident::new(&tablename_titlecase, Span::call_site());
        Self {
            query_fn_name: Ident::new(&format!("get_all_{tablename}"), Span::call_site()),
            query: format!("SELECT * FROM {tablename}"),
            path: None,
            bind: None,
            fetch_mode: quote! {.fetch_all(&state.db)},
            querytype: quote! {sqlx::query_as::<_, #structname>},
            response: quote! {Ok((StatusCode::OK, Json(res)))},
            declaration: quote! {let res = match },
            error_handling: quote! {Ok(res) => res,
        Err(e) => return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string()
            ))}
    }
        }

    fn get_one(tablename: &str) -> Self {
    let mut v: Vec<char> = tablename.chars().collect();
    v[0] = v[0].to_uppercase().next().unwrap();
    let tablename_titlecase: String = v.into_iter().collect();

    let structname = Ident::new(&tablename_titlecase, Span::call_site());
        Self {
            query_fn_name: Ident::new(&format!("get_{tablename}_by_id"), Span::call_site()),
            query: format!("SELECT * FROM {tablename} WHERE id = $1"),
            path: Some(quote! {Path(id): Path<i32>}),
            bind: Some(quote! {.bind(id)}),
            fetch_mode: quote! {.fetch_one(&state.db)},
            querytype: quote! {sqlx::query_as::<_, #structname>},
            response: quote! {Ok((StatusCode::OK, Json(res)))},
            declaration: quote! {let res = match },
            error_handling: quote! {Ok(res) => res,
        Err(e) => return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string()
            ))}
        }
    }

    fn create(tablename: &str) -> Self {
        Self {
            query_fn_name: Ident::new(&format!("create_{tablename}"), Span::call_site()),
            query: format!("INSERT INTO {tablename} () VALUES ()"),
            path: None,
            bind: None,
            fetch_mode: quote! {.execute(&state.db)},
            querytype: quote! {sqlx::query},
            response: quote! {Ok(StatusCode::CREATED)},
            declaration: quote! {if let Err(e) = },
            error_handling: quote! {return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string()
            ))}
        }
    }

    fn update(tablename: &str) -> Self {
        Self {
            query_fn_name: Ident::new(&format!("update_{tablename}_by_id"), Span::call_site()),
            query: format!("UPDATE {tablename} set $1 = $2 WHERE id = $3"),
            path: Some(quote! {Path(id): Path<i32>}),
            bind: Some(quote! {.bind(id)}),
            fetch_mode: quote! {.fetch_all(&state.db)},
            querytype: quote! {sqlx::query},
            response: quote! {Ok(StatusCode::OK)},
            declaration: quote! {if let Err(e) = },
            error_handling: quote! {return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string()
            ))}
        }
    }

    fn delete(tablename: &str) -> Self {
        Self {
            query_fn_name: Ident::new(&format!("delete_{tablename}_by_id"), Span::call_site()),
            query: format!("SELECT * FROM {tablename}"),
            path: Some(quote! {Path(id): Path<i32>}),
            bind: Some(quote! {.bind(id)}),
            fetch_mode: quote! {.fetch_all(&state.db)},
            querytype: quote! {sqlx::query},
            response: quote! {Ok(StatusCode::OK)},
            declaration: quote! {if let Err(e) = },
            error_handling: quote! {return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string()
            ))}
        }
    }

    pub fn create_query_data(tablename: &str) -> Vec<Self> {
        vec![
            QueryGen::get_all(tablename),
            QueryGen::get_one(tablename),
            QueryGen::create(tablename),
            QueryGen::update(tablename),
            QueryGen::delete(tablename),
        ]
    }

}