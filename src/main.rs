use serenity::client::Client;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};
use serenity::builder::*;

use colored::*;

use std::process::exit;

use nikel_rs::*;

mod config;

#[group]
#[commands(ping, echo, courses, textbooks, exams, evals, food, food, services, buildings, parking)]
struct General;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{}{}", "Ready! Username: ".green(), ready.user.name);
    }
}

fn main() {

    let token: String = match config::get_token() {
        Ok(tok) => {
            println!("{}", "Got token".green());
            tok
        },
        _ => {
            eprintln!("{}", "Couldn't get bot token".red());
            exit(1);
        }
    };

    match serenity::client::validate_token(&token) {
        Ok(()) => {
            println!("{}", "Token validated".green());
        },
        _ => {
            eprintln!("{}", "Couldn't validate token".red());
            exit(1);
        }
    }

    // Login with a bot token from the environment
    let mut client = Client::new(&token, Handler)
        .expect(&"Error creating client".red());
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(".")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("{}{:?}", "An error occurred while running the client: ".red(), why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "Pong!")?;

    return Ok(());
}

#[command]
fn echo(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, msg.content_safe(&ctx).replace(".echo ", ""))?;

    return Ok(());
}

//type NikelFunc<T> = fn(&NikelAPI, &Parameters) -> NikelResult<T>;

fn req<T: Clone>(ctx: &Context, msg: &Message, nik: NikelResult<T>, proc: fn(T, &mut CreateEmbed)) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| {
        match nik {
            Ok(resp) => {
                if resp.response.len() == 0 {
                    m.embed(|e: &mut serenity::builder::CreateEmbed| {
                        e.colour((242, 170, 0)).title("No Results").description("No results were returned by the API")
                    })
                } else {
                    m.embed(|e: &mut serenity::builder::CreateEmbed| {
                        e.color((0, 46, 100));
                        proc(resp.response[0].clone(), e);
                        e
                    })
                }
            },
            _ => {
                m.embed(|e: &mut serenity::builder::CreateEmbed| {
                    e.colour((200, 100, 100)).title("Failed").description("There was a problem with that")
                })
            }
        }
    })?;
    Ok(()) as CommandResult
}

#[command]
fn courses(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Course>(&ctx, msg, client.courses(to_params(&msg.content_safe(&ctx))), |c: Course, m: &mut CreateEmbed| {
        let title = format!("{}{}", c.code.expect("No course code!?"),
            match c.name {
                Some(name) => format!(" - {}", name),
                _ => "".to_owned()
            }
        );
        m.title(title)
         .field("Campus", c.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Term", c.term.unwrap_or("Unavailable".to_owned()), true)
         .field("UTM Dist. Req.", c.utm_distribution.unwrap_or("Unavailable".to_owned()), true)
         .field("Prereqs", c.prerequisites.unwrap_or("Unavailable".to_owned()), true)
         .field("Coreqs", c.corequisites.unwrap_or("Unavailable".to_owned()), true)
         .field("Exclusions", c.exclusions.unwrap_or("Unavailable".to_owned()), true)
         .field("Description", c.description.unwrap_or("Unavailable".to_owned()), false);
    })
}

#[command]
fn textbooks(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Textbook>(&ctx, msg, client.textbooks(to_params(&msg.content_safe(&ctx))), |t: Textbook, m: &mut CreateEmbed| {
        m.title(t.title.unwrap_or("Textbook".to_owned()))
         .field("Price", format!("${}", t.price.unwrap_or(-1.0)), true)
         .field("ISBN", t.isbn.unwrap_or("Unavailable".to_owned()), true)
         .field("Courses", t.courses.into_iter().map(|c| c.code.unwrap_or("Unavailable".to_owned()).to_owned()).collect::<Vec<_>>().join("\n"), false);
    })
}

#[command]
fn exams(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Exam>(&ctx, msg, client.exams(to_params(&msg.content_safe(&ctx))), |e: Exam, m: &mut CreateEmbed| {
        m.title("Exam")
         .field("Course", e.course_code.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", e.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Date", e.date.unwrap_or("Unavailable".to_owned()), true)
         .field("Start", e.start.unwrap_or(0), true)
         .field("End", e.end.unwrap_or(0), true);
    })
}

#[command]
fn evals(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Eval>(&ctx, msg, client.evals(to_params(&msg.content_safe(&ctx))), |e: Eval, m: &mut CreateEmbed| {
        m.title("Eval")
         .field("Name", e.name.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", e.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Last Updated", e.last_updated.unwrap_or("Unavailable".to_owned()), true);
    })
}

#[command]
fn food(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Food>(&ctx, msg, client.food(to_params(&msg.content_safe(&ctx))), |f: Food, m: &mut CreateEmbed| {
        m.title(f.name.unwrap_or("Food".to_owned()))
         .field("Campus", f.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Address", f.address.unwrap_or("Unavailable".to_owned()), true)
         .field("Tags", f.tags.unwrap_or("Unavailable".to_owned()), true)
         .field("URL", f.url.unwrap_or("Unavailable".to_owned()), true);
         match f.image {
             Some(url) => { m.image(url); },
             _ => {}
         }
    })
}

#[command]
fn services(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Service>(&ctx, msg, client.services(to_params(&msg.content_safe(&ctx))), |s: Service, m: &mut CreateEmbed| {
        m.title("Service")
         .field("Name", s.name.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", s.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("tags", s.tags.unwrap_or("Unavailable".to_owned()), true)
         .field("Building", s.building_id.unwrap_or("Unavailable".to_owned()), true);
         match s.image {
             Some(url) => { m.image(url); },
             _ => {}
         }
    })
}

#[command]
fn buildings(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Building>(&ctx, msg, client.buildings(to_params(&msg.content_safe(&ctx))), |b: Building, m: &mut CreateEmbed| {
        m.title(b.name.unwrap_or("Building".to_owned()))
         .field("Address", format!("{},{},{}", b.address.street.unwrap_or("?".to_owned()), b.address.city.unwrap_or("?".to_owned()), b.address.country.unwrap_or("?".to_owned())), true)
         .field("Coordinates", format!("{} degrees North, {} degrees East", b.coordinates.latitude.unwrap_or(0.0), b.coordinates.longitude.unwrap_or(0.0)), true);
    })
}

#[command]
fn parking(ctx: &mut Context, msg: &Message) -> CommandResult {
    let client = NikelAPI::new();
    req::<Parking>(&ctx, msg, client.parking(to_params(&msg.content_safe(&ctx))), |p: Parking, m: &mut CreateEmbed| {
        m.title("Parking")
         .field("Name", p.name.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", p.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Address", p.address.unwrap_or("Unavailable".to_owned()), true)
         .field("Tags", p.description.unwrap_or("Unavailable".to_owned()), false);
    })
}

fn to_params(input: &String) -> Parameters {
    split_once(input, ' ').ok().unwrap().1.split(',')
        .map(|arg| arg.split(":").map(|e| e.trim()).collect())
        .filter(|v: &Vec<&str>| {
            if v.len() != 2 {
                println!("Couldn't parse option {:?}, ignoring", v);
                false
            } else {
                true
            }
        })
        .map(|v: Vec<&str>| (v[0], v[1]))
        .collect()
}

fn split_once(in_string: &str, delim: char) -> Result<(&str, &str), ()> {
    let mut splitter = in_string.splitn(2, delim);
    let first = match splitter.next() {
        Some(s) => s,
        _ => return Err(())
    };
    let second = match splitter.next() {
        Some(s) => s,
        _ => return Err(())
    };
    Ok((first, second))
}
