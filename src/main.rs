use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde;
use serde::Deserialize;
use serde_json::json;
use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref CODE_REGEX: Regex = Regex::new(r"^`(?:(``)(.*\n)?)?((?:.*\n?)*?)`\1?$").unwrap();
}

#[group]
#[commands(goodpractices, play, eval)]
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

#[command]
fn play(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if let Some(code) = parse_code_arg(&args) {
        msg.react(&ctx, '🔄')?;
        let output = match run_playground(code) {
            Ok(res) => {
                if res.success {
                    ctx.http.delete_reaction(
                        msg.channel_id.into(),
                        msg.id.into(),
                        Some(ctx.http.get_current_user()?.id.into()),
                        &'🔄'.into(),
                    )?;
                    msg.react(&ctx, '✅')?;
                    res.stdout
                } else {
                    ctx.http.delete_reaction(
                        msg.channel_id.into(),
                        msg.id.into(),
                        Some(ctx.http.get_current_user()?.id.into()),
                        &'🔄'.into(),
                    )?;
                    msg.react(&ctx, '❌')?;
                    res.stderr
                }
            }
            Err(e) => format!("Failed to run code through playground: {:?}", e),
        };
        format_code_output(&output, code)
    } else {
        "Please provide code to be compiled and run.".to_string()
    };
    msg.channel_id.say(&ctx, say_content)?;

    Ok(())
}

#[command]
fn eval(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if let Some(code) = parse_code_arg(&args) {
        msg.react(&ctx, '🔄')?;
        let code = format!(r#"fn main() {{println!("{{:?}}", {});}}"#, code);
        let output = match run_playground(&code) {
            Ok(res) => {
                if res.success {
                    ctx.http.delete_reaction(
                        msg.channel_id.into(),
                        msg.id.into(),
                        Some(ctx.http.get_current_user()?.id.into()),
                        &'🔄'.into(),
                    )?;
                    msg.react(&ctx, '✅')?;
                    res.stdout
                } else {
                    ctx.http.delete_reaction(
                        msg.channel_id.into(),
                        msg.id.into(),
                        Some(ctx.http.get_current_user()?.id.into()),
                        &'🔄'.into(),
                    )?;
                    msg.react(&ctx, '❌')?;
                    res.stderr
                }
            }
            Err(e) => format!("Failed to run code through playground: {:?}", e),
        };
        format_code_output(&output, &code)
    } else {
        "Please provide code to be compiled and run.".to_string()
    };
    msg.channel_id.say(&ctx, say_content)?;

    Ok(())
}

fn parse_code_arg(args: &Args) -> Option<&str> {
    println!("{}", args.message());
    if let Some(captures) = CODE_REGEX.captures(args.message()).unwrap() {
        let a = captures.get(2).map_or("", |x| x.as_str());
        let b = captures.get(3).map_or("", |x| x.as_str());
        Some(if b == "" { a } else { b })
    } else {
        None
    }
}

fn format_code_output(output: &str, code: &str) -> String {
    if output.len() > 1994 {
        let url: String = format!(
            "https://play.rust-lang.org?{}",
            url::form_urlencoded::Serializer::new(String::new())
                .append_pair("code", code)
                .finish()
        );
        let output_trunc = output.chars().take(1991 - url.len()).collect::<String>();
        format!("```{}...```{}", output_trunc, url)
    } else {
        format!("```{}```", output)
    }
}

#[derive(Deserialize)]
struct PlaygroundResponse {
    success: bool,
    stdout: String,
    stderr: String,
}

fn run_playground(code: &str) -> Result<PlaygroundResponse, reqwest::Error> {
    let res = REQWEST_CLIENT
        .post("https://play.rust-lang.org/execute")
        .json(&json!({
            "channel": "nightly",
            "edition": "2018",
            "code": code,
            "crateType": "bin",
            "mode": "debug",
            "tests": false
        }))
        .send()?;
    res.json::<PlaygroundResponse>()
}
