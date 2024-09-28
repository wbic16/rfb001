use libphext::phext;
use axum::{routing::get, Router, response::Html};
use std::net::SocketAddr;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::futures::StreamExt;
use std::env;
use std::collections::HashSet;
use chrono::{Utc, Datelike, Duration};


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

#[poise::command(slash_command, prefix_command)]
async fn get_weekly_updates(ctx: Context<'_>) -> Result<(), Error> {
    let channel_name = "weekly-updates";
    
    // Get a clone of the Guild object, which is Send
    let guild = ctx.guild().ok_or("This command must be used in a server")?.clone();
    
    let channel = guild
        .channels
        .values()
        .find(|c| c.name == channel_name)
        .ok_or(format!("Could not find #{} channel", channel_name))?;

    let http = &ctx.http(); // Store the reference to ctx.http()
    let now = Utc::now();
    let days_since_sunday = now.weekday().num_days_from_sunday() + 1;
    let last_sunday = (now - Duration::days(days_since_sunday as i64)).into();
    let today = now.into();

    let mut users = HashSet::new();
    let mut messages = channel.id.messages_iter(http).boxed(); // Use the stored http reference
    while let Some(message_result) = messages.next().await {
        match message_result {
            Ok(message) => {
                if message.timestamp < last_sunday {
                    break; // Stop iterating if we've reached messages older than last Sunday
                }
                if message.timestamp <= today && !message.author.bot {
                    users.insert(message.author.name.clone());
                }
            },
            Err(error) => {
                ctx.say(format!("Error fetching messages: {}", error)).await?;
                return Ok(());
            }
        }
    }

    let response = if users.is_empty() {
        "No users posted in #weekly-updates last week.".to_string()
    } else {
        format!("Users who posted in #weekly-updates last week:\n{}", users.into_iter().collect::<Vec<_>>().join("\n"))
    };

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
    dotenv::dotenv().ok();
    let token = env::var("RFB001_TOKEN").expect("Set RFB001_TOKEN to your API token.");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                get_weekly_updates()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Err(e) = poise::builtins::register_globally(ctx, &framework.options().commands).await {
                    println!("Error registering commands globally: {}", e);
                } else {
                    println!("Commands registered globally");
                }
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
