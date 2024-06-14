use std::marker::PhantomData;

use poem::{
    endpoint::EmbeddedFileEndpoint, listener::Listener, middleware::Tracing, Endpoint, EndpointExt,
    Request, Response, Route, Server,
};
use rust_embed::Embed;

const INDEX_HTML: &str = "index.html";

#[derive(Embed)]
#[folder = "../../app/dist/"]
struct App;

fn app() -> impl Endpoint {
    Route::new()
        .nest("/", SPAEndpoint::<App>::default())
        .with(Tracing)
}

struct SPAEndpoint<E: Embed + Send + Sync> {
    _embed: PhantomData<E>,
}

impl<E: Embed + Send + Sync> Default for SPAEndpoint<E> {
    fn default() -> Self {
        Self {
            _embed: Default::default(),
        }
    }
}

impl<E: Embed + Send + Sync> Endpoint for SPAEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output, poem::Error> {
        let mut path = req
            .uri()
            .path()
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();
        if path.is_empty() {
            path = INDEX_HTML.to_string();
        }
        let path = path.as_ref();
        if E::get(path).is_some() {
            EmbeddedFileEndpoint::<E>::new(path).call(req).await
        } else {
            EmbeddedFileEndpoint::<E>::new(INDEX_HTML).call(req).await
        }
    }
}

pub(crate) async fn start<L>(listener: L) -> eyre::Result<()>
where
    L: Listener,
    L::Acceptor: 'static,
{
    Server::new(listener).run(app()).await?;
    Ok(())
}
