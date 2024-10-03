use libphext::phext;
use axum::{routing::get, Router, response::Html};
use std::net::SocketAddr;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::futures::StreamExt;
use serenity::builder::EditMessage;
use std::env;
use std::collections::HashMap;
use chrono::{Utc, Datelike, DateTime, Duration};
use serde_json::json;
use tokio::fs;

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
async fn fetch_weekly_updates(ctx: Context<'_>) -> Result<(), Error> {
    let channel_name = "weekly-updates";
    let leaderboard_channel_name = "leaderboard";
    
    // Get a clone of the Guild object, which is Send
    let guild = ctx.guild().ok_or("This command must be used in a server")?.clone();
    
    let channel = guild
        .channels
        .values()
        .find(|c| c.name == channel_name)
        .ok_or(format!("Could not find #{} channel", channel_name))?;

    // Find the leaderboard channel
    let leaderboard_channel = guild
        .channels
        .values()
        .find(|c| c.name == leaderboard_channel_name)
        .ok_or("Couldn't find leaderboard channel")?;

    let http = &ctx.http(); // Store the reference to ctx.http()
    let now = Utc::now();
    let days_since_sunday = now.weekday().num_days_from_sunday() + 1;
    let last_sunday = (now - Duration::days(days_since_sunday as i64)).into();
    let today = now.into();

    //let mut user_messages: HashMap<String, Vec<String>> = HashMap::new();
    //
    //check if json file exists and update user_messages with the values
    //if not create a new file (this allows storage of messages that cannot be read due to the
    //message history limit)
    let mut user_messages: HashMap<String, Vec<String>> = match fs::read_to_string("weekly_updates.json").await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| HashMap::new()),
        Err(_) => HashMap::new(), // If file doesn't exist, create a new map
    };
    
    let mut messages = channel.id.messages_iter(http).boxed(); // Use the stored http reference
    while let Some(message_result) = messages.next().await {
        match message_result {
            Ok(message) => {
                if message.timestamp < last_sunday {
                    break; // Stop iterating if we've reached messages older than last Sunday
                }
                if message.timestamp <= today && !message.author.bot && message.content.contains("x.com") {
                    let username = message.author.name.clone();
                    let timestamp = message.timestamp.to_string();

                    let last_message_time = user_messages.get(&username)
                        .and_then(|timestamps| timestamps.last())
                        .map(|ts| ts.parse::<DateTime<Utc>>().ok())
                        .flatten();

                    if let Some(last_ts) = last_message_time {
                        // check if message timestamp and last message timestamp have same date
                        if last_ts.date_naive() == message.timestamp.date_naive() { continue; }
                    }

                    user_messages
                        .entry(username)
                        .or_insert_with(Vec::new)
                        .push(timestamp);
                }
            },
            Err(error) => {
                ctx.say(format!("Error fetching messages: {}", error)).await?;
                return Ok(());
            }
        }
    }
    
    let json_data = json!(user_messages);
    fs::write("weekly_updates.json", serde_json::to_string_pretty(&json_data).unwrap()).await?;
    ctx.say("weekly_updates.json has been updated successfully.").await?;

    let mut user_points: Vec<(String, usize)> = user_messages
        .iter()
        .map(|(user, timestamps)| (user.clone(), timestamps.len())) // Count the length of the timestamp array for each user
        .collect();

    // Sort users by points in descending order
    user_points.sort_by(|a, b| b.1.cmp(&a.1));
   
    // generate leaderboard message
    let mut leaderboard_message = String::from("**Builder Rankings:**\n\n");
    for (rank, (user, points)) in user_points.iter().enumerate() {
        leaderboard_message.push_str(&format!("**[{}] {}** — {} points\n", rank + 1, user, points));
    }

    let mut leaderboard_messages = leaderboard_channel.id.messages_iter(http).boxed();
    if let Some(last_message_result) = leaderboard_messages.next().await {
        if let Ok(mut last_message) = last_message_result {
            // Edit the last message with the updated leaderboard
            let edit_message = EditMessage::new().content(&leaderboard_message);
            last_message.edit(http, edit_message).await?;
        } else {
            // If no last message, post a new message
            leaderboard_channel.id.say(http, &leaderboard_message).await?;
        }
    } else {
        // If no last message, post a new message
        leaderboard_channel.id.say(http, &leaderboard_message).await?;
    }
    
    ctx.say("Leaderboard updated successfully!").await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // Create a simple route handler
    //let app = Router::new()
    //    .route("/", get(index))
    //    .route("/api/v1/leaders", get(leaders))
    //    .route("/api/v1/scan", get(scan))
    //;
    //
    //// Define the address to serve on
    //let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    //println!("Serving on http://{}", addr);
    //
    //// Run the web server with axum_server
    //axum_server::bind(addr)
    //.serve(app.into_make_service())
    //.await
    //.unwrap();
    //
    println!("Starting Bot...");
    scan().await;
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
                fetch_weekly_updates()
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
