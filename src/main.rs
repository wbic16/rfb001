use libphext::phext;
use axum::{routing::get, Router, response::Html};
use std::net::SocketAddr;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn ping(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    

    // Create a simple route handler
    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/leaders", get(leaders))
        .route("/api/v1/scan", get(scan))
    ;

    // Define the address to serve on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);

    // Run the web server with axum_server
    axum_server::bind(addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: poise::serenity_prelude::Context, msg: Message) {
        if !msg.author.bot {
            let summary = phext::fetch(&msg.content, phext::to_coordinate("1.1.1/1.1.1/1.1.1"));
            println!("New message in {}: {}", msg.channel_id, summary);
        }
    }
}


async fn index() -> Html<&'static str> {
    Html("<html><head>
    <style type='text/css'>
    body { background-color: #151515; color: #e0ffe0; font-family: monospace; }
    a, a:visited { color: white; }
    a:hover, a:visited:hover { color: grey; }
    </style>
    </head><body>
    <ul>
    <li><a href='/api/v1/leaders'>/api/v1/leaders</a></li>
    <li><a href='/api/v1/scan'>/api/v1/scan</a></li>
    </ul></body></html>")
}

async fn leaders() -> &'static str {
    "@wbic16"
}

async fn scan() -> &'static str {
    let token = env::var("RFB001_TOKEN").expect("Set RFB001_TOKEN to your API token.");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();
    
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    let _result = client.unwrap().start().await.expect("Serenity Error");
    "request completed"
}