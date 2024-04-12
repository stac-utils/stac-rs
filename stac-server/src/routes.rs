//! Routes for serving API endpoints.

use crate::{Api, Backend, APPLICATION_GEO_JSON, APPLICATION_OPENAPI_3_0};
use axum::{
    extract::{rejection::JsonRejection, Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::Method;
use stac_api::{GetItems, GetSearch, Items, Search};

/// Creates an [axum::Router] from an [Api].
///
/// # Examples
///
/// ```
/// use stac_server::{Api, MemoryBackend, routes};
///
/// let api = Api::new(MemoryBackend::new(), "http://stac.test").unwrap();
/// let router = routes::from_api(api);
/// ```
pub fn from_api<B: Backend>(api: Api<B>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api", get(service_desc))
        .route("/api.html", get(service_doc))
        .route("/conformance", get(conformance))
        .route("/collections", get(collections))
        .route("/collections/:collection_id", get(collection))
        .route("/collections/:collection_id/items", get(items))
        .route("/collections/:collection_id/items/:item_id", get(item))
        .route("/search", get(get_search))
        .route("/search", post(post_search))
        .with_state(api)
}

/// Returns the `/` endpoint from the [core conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/core#endpoints).
pub async fn root<B: Backend>(State(api): State<Api<B>>) -> Response {
    match api.root().await {
        Ok(root) => Json(root).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
    }
}

/// Returns the `/api` endpoint from the [core conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/core#endpoints).
pub async fn service_desc() -> Response {
    // The OpenAPI definition is completely stolen from [stac-server](https://github.com/stac-utils/stac-server/blob/dd7e3acbf47485425e2068fd7fbbceeafe4b4e8c/src/lambdas/api/openapi.yaml).
    //
    // TODO add a script to update the definition in this library.
    (
        [(CONTENT_TYPE, APPLICATION_OPENAPI_3_0)],
        include_str!("openapi.yaml"),
    )
        .into_response()
}

/// Returns the `/api.html` endpoint from the [core conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/core#endpoints).
pub async fn service_doc() -> Response {
    // The redoc file is completely stolen from [stac-server](https://github.com/stac-utils/stac-server/blob/dd7e3acbf47485425e2068fd7fbbceeafe4b4e8c/src/lambdas/api/redoc.html).
    Html(include_str!("redoc.html")).into_response()
}

/// Returns the `/conformance` endpoint from the [ogcapi-features conformance
/// class](https://github.com/radiantearth/stac-api-spec/blob/release/v1.0.0/ogcapi-features/README.md#endpoints).
pub async fn conformance<B: Backend>(State(api): State<Api<B>>) -> Response {
    Json(api.conformance()).into_response()
}

/// Returns the `/collections` endpoint from the [ogcapi-features conformance
/// class](https://github.com/radiantearth/stac-api-spec/blob/release/v1.0.0/ogcapi-features/README.md#endpoints).
pub async fn collections<B: Backend>(State(api): State<Api<B>>) -> Response {
    match api.collections().await {
        Ok(collections) => Json(collections).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
    }
}

/// Returns the `/collections/{collectionId}` endpoint from the [ogcapi-features
/// conformance
/// class](https://github.com/radiantearth/stac-api-spec/blob/release/v1.0.0/ogcapi-features/README.md#endpoints).
pub async fn collection<B: Backend>(
    State(api): State<Api<B>>,
    Path(collection_id): Path<String>,
) -> Response {
    match api.collection(&collection_id).await {
        Ok(option) => {
            if let Some(collection) = option {
                Json(collection).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    format!("no collection with id='{}'", collection_id),
                )
                    .into_response()
            }
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
    }
}

/// Returns the `/collections/{collectionId}/items` endpoint from the
/// [ogcapi-features conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/ogcapi-features#collection-items-collectionscollectioniditems)
pub async fn items<B: Backend>(
    State(api): State<Api<B>>,
    Path(collection_id): Path<String>,
    items: Query<GetItems>,
) -> Response {
    match Items::try_from(items.0).and_then(Items::valid) {
        Ok(items) => match api.items(&collection_id, items).await {
            Ok(option) => {
                if let Some(items) = option {
                    let mut response = Json(items).into_response();
                    let _ = response
                        .headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_GEO_JSON));
                    response
                } else {
                    (
                        StatusCode::NOT_FOUND,
                        format!("no collection with id='{}'", collection_id),
                    )
                        .into_response()
                }
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
        },
        Err(err) => (StatusCode::BAD_REQUEST, format!("invalid query: {}", err,)).into_response(),
    }
}

/// Returns the `/collections/{collectionId}/items/{itemId}` endpoint from the
/// [ogcapi-features conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/ogcapi-features#collection-items-collectionscollectioniditems)
pub async fn item<B: Backend>(
    State(api): State<Api<B>>,
    Path((collection_id, item_id)): Path<(String, String)>,
) -> Response {
    match api.item(&collection_id, &item_id).await {
        Ok(option) => {
            if let Some(item) = option {
                let mut response = Json(item).into_response();
                let _ = response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_GEO_JSON));
                response
            } else {
                (
                    StatusCode::NOT_FOUND,
                    format!(
                        "no item with id='{}' in collection='{}'",
                        item_id, collection_id
                    ),
                )
                    .into_response()
            }
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
    }
}

/// Returns the GET `/search` endpoint from the [item search conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/item-search)
pub async fn get_search<B: Backend>(
    State(api): State<Api<B>>,
    search: Query<GetSearch>,
) -> Response {
    match Search::try_from(search.0).and_then(Search::valid) {
        Ok(search) => match api.search(search, Method::GET).await {
            Ok(item_collection) => {
                let mut response = Json(item_collection).into_response();
                let _ = response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_GEO_JSON));
                response
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
        },
        Err(err) => (StatusCode::BAD_REQUEST, format!("invalid query: {}", err,)).into_response(),
    }
}

/// Returns the POST `/search` endpoint from the [item search conformance
/// class](https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/item-search)
pub async fn post_search<B: Backend>(
    State(api): State<Api<B>>,
    search: Result<Json<Search>, JsonRejection>,
) -> Response {
    match search
        .map_err(|err| err.to_string())
        .and_then(|search| search.0.valid().map_err(|err| err.to_string()))
    {
        Ok(search) => match api.search(search, Method::POST).await {
            Ok(item_collection) => {
                let mut response = Json(item_collection).into_response();
                let _ = response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_GEO_JSON));
                response
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)).into_response(),
        },
        Err(err) => (StatusCode::BAD_REQUEST, format!("invalid query: {}", err,)).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Api, Backend, MemoryBackend};
    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Request, Response, StatusCode},
    };
    use stac::{Collection, Item};
    use tower::util::ServiceExt;

    async fn get(backend: MemoryBackend, uri: &str) -> Response<Body> {
        let router = super::from_api(
            Api::new(backend, "http://stac.test/")
                .unwrap()
                .id("an-id")
                .description("a description"),
        );
        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn post(backend: MemoryBackend, uri: &str) -> Response<Body> {
        let router = super::from_api(
            Api::new(backend, "http://stac.test/")
                .unwrap()
                .id("an-id")
                .description("a description"),
        );
        router
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body("{}".to_string())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn root() {
        let response = get(MemoryBackend::new(), "/").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn service_description() {
        let response = get(MemoryBackend::new(), "/api").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/vnd.oai.openapi+json;version=3.0"
        );
    }

    #[tokio::test]
    async fn service_doc() {
        let response = get(MemoryBackend::new(), "/api.html").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn conformance() {
        let response = get(MemoryBackend::new(), "/conformance").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn collections() {
        let response = get(MemoryBackend::new(), "/collections").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn collection() {
        let response = get(MemoryBackend::new(), "/collections/an-id").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "A description"))
            .await
            .unwrap();
        let response = get(backend, "/collections/an-id").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn items() {
        let response = get(MemoryBackend::new(), "/collections/collection-id/items").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("collection-id", "A description"))
            .await
            .unwrap();
        backend
            .add_item(Item::new("item-id").collection("collection-id"))
            .await
            .unwrap();
        let response = get(backend, "/collections/collection-id/items").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }

    #[tokio::test]
    async fn item() {
        let response = get(
            MemoryBackend::new(),
            "/collections/collection-id/items/item-id",
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("collection-id", "A description"))
            .await
            .unwrap();
        backend
            .add_item(Item::new("item-id").collection("collection-id"))
            .await
            .unwrap();
        let response = get(backend, "/collections/collection-id/items/item-id").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }

    #[tokio::test]
    async fn get_search() {
        let response = get(MemoryBackend::new(), "/search").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }

    #[tokio::test]
    async fn post_search() {
        let response = post(MemoryBackend::new(), "/search").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }
}
