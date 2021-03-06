use std::{str::FromStr, collections::HashMap};
use anyhow::{anyhow,Result};
use colored::*;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use clap::{AppSettings, Clap};

// define the CLI entrance of HTTPie, including several sub commands

/// A naive httpie implimentation with Rust.
#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "shang <shang.gong@outlook.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

// 子命令对应不同的 HTTP 方法，暂支持 Get & Post
#[derive(Clap, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
    // TODO: other HTTP methods not supported yet.
}

/// feed get with an url and we will retrieve the response for you
#[derive(Clap, Debug)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

/// feed post with an url and optional key=val pairs. We will post the data 
/// as JSON, and retrieve the response for you
#[derive(Clap, Debug)]
struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair>,
}

#[derive(Debug)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split using "="
        let mut split = s.split("=");
        let err = || anyhow!(format!("Faild to parse {}", s));

        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> { 
    Ok(s.parse()?)
}

fn parse_url(s: &str) -> Result<String> { 
    let _url: Url = s.parse()?;

    Ok(s.into())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    
    let client = Client::new();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!("");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v.subtype() == mime::JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }

        _ => {
            println!("{}", body)
        }
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers().get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("http://example.com").is_ok());
        assert!(parse_url("https://example.xyz").is_ok());
        assert!(parse_url("abc").is_err());
    }
}