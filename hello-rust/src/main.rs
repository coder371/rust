use async_graphql::{http::GraphiQLSource, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{response::Html, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};

#[derive(SimpleObject)]
struct User {
    id: i32,
    name: String,
}

#[derive(SimpleObject)]
struct Product {
    id: i32,
    name: String,
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn users(&self) -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Ahmed".to_string(),
            },
            User {
                id: 2,
                name: "Sara".to_string(),
            },
        ]
    }

    async fn products(&self) -> Vec<Product> {
        vec![Product {
            id: 1,
            name: "Rust Book".to_string(),
        }]
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, name: String) -> User {
        User { id: 3, name }
    }

    async fn create_product(&self, name: String) -> Product {
        Product { id: 2, name }
    }
}

type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

async fn graphql_handler(schema: axum::extract::State<AppSchema>, request: GraphQLRequest) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();
    let app = Router::new()
        .route("/graphql", post(graphql_handler).get(graphiql))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS])
                .allow_headers(Any)
                .allow_private_network(true),
        )
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("failed to bind port 8000");
    println!("GraphQL server: http://127.0.0.1:8000/graphql");
    axum::serve(listener, app).await.expect("server failed");
}
