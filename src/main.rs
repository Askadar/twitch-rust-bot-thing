use std::collections::HashMap;

use redis::Commands;
use tokio;
use twitch_api::twitch_oauth2::{AccessToken, UserToken};
use twitch_api::HelixClient;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{irc, ClientConfig, SecureTCPTransport, TwitchIRCClient};

async fn handle_message(
    message: ServerMessage,
    r: &mut redis::Client,
    login_live_hash: &HashMap<String, String>,
) -> () {
    match message {
        ServerMessage::Privmsg(m) => {
            r.sadd::<String, String, ()>(
                format!(
                    "messages:{}:{}",
                    m.channel_login,
                    login_live_hash.get(&m.channel_login).unwrap()
                ),
                format!("{}: {}", m.sender.name, m.message_text),
            )
            .unwrap();
        }
        ServerMessage::Join(m) => {
            let added = r
                .sadd::<String, String, usize>(
                    format!(
                        "joins:{}:{}",
                        m.channel_login,
                        login_live_hash.get(&m.channel_login).unwrap()
                    ),
                    m.user_login.clone(),
                )
                .unwrap();
            if added == 1 {
                r.zincr::<_, _, _, _>(format!("watchStreaks:{}", m.channel_login), m.user_login, 1)
                    .unwrap()
            }
        }
        // ServerMessage::Part(m) => {
        //     print!("\n#!# {} left {} #!# ", m.user_login, m.channel_login);
        //     std::io::stdout().flush().unwrap();
        // }
        _ => {}
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let mut r = redis::Client::open("redis://127.0.0.1/").unwrap();

    let login_name = std::env::var("BOT_USERNAME").expect("Missing env 'BOT_USERNAME'");
    let oauth_token = std::env::var("BOT_TOKEN").expect("Missing env 'BOT_TOKEN'");

    let channels: Vec<String> = r.lrange("channels", 0, -1).unwrap();

    let t: HelixClient<reqwest::Client> = HelixClient::default();

    let token = UserToken::from_token(&t, AccessToken::from(oauth_token.clone()))
        .await
        .unwrap();

    let config =
        ClientConfig::new_simple(StaticLoginCredentials::new(login_name, Some(oauth_token)));

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let channels_info = t
        .req_get(
            twitch_api::helix::streams::GetStreamsRequest::user_logins(
                channels
                    .iter()
                    .map(|a| twitch_api::types::NicknameRef::from_str(a))
                    .collect::<Vec<_>>(),
            ),
            &token,
        )
        .await
        .unwrap()
        .data;

    let login_live_hash: HashMap<String, String> = channels_info
        .iter()
        .map(|s| (s.user_login.clone().to_string(), s.id.clone().to_string()))
        .collect();

        // tokio::spawn
    let join_handle = tokio::spawn(async move {
        println!("\"Threading\"");
        while let Some(message) = incoming_messages.recv().await {
            handle_message(message, &mut r, &login_live_hash).await;
        }
    });

    channels_info.iter().for_each(|channel| {
        client
            .join(channel.user_login.to_string().to_owned())
            .unwrap()
    });
    client
        .send_message(irc!["CAP REQ", "twitch.tv/membership"])
        .await
        .unwrap();

    println!("{:?}", channels_info);
    println!(
        "Prepped on {:?}",
        channels_info
            .iter()
            .map(|s| s.user_login.to_string())
            .collect::<Vec<_>>()
    );

    join_handle.await.unwrap();
}
