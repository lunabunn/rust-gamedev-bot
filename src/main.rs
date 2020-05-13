use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};

#[group]
#[commands(goodpractices)]
struct General;

use std::env;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("?"))
            .group(&GENERAL_GROUP),
    );

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn goodpractices(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |x| {
        x.embed(|embed| {
            embed
                .description(include_str!("../data/good_practices.md"))
                .timestamp(msg.timestamp.to_rfc3339())
                .footer(|footer| {
                    footer
                        .icon_url(
                            msg.author
                                .avatar_url()
                                .unwrap_or(msg.author.default_avatar_url().to_owned()),
                        )
                        .text(format!("Requested by {}", msg.author.name))
                })
        })
    })?;

    Ok(())
}
