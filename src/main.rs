use serenity::client::Client;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    Args,
    HelpOptions,
    CommandGroup,
    help_commands,
    macros::*
};
use serenity::builder::*;

use std::collections::HashSet;

use colored::*;

use std::process::exit;

use nikel_rs::*;

mod config;

mod util;

const AC_COURSES: &str = "https://fas.calendar.utoronto.ca/course";

#[group]
#[commands(courses, textbooks, exams, evals, food, food, services, buildings, parking)]
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
        .group(&GENERAL_GROUP)
        .help(&HELP));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("{}{:?}", "An error occurred while running the client: ".red(), why);
    }
}

#[help]
#[strikethrough_commands_tip_in_guild("")]
#[strikethrough_commands_tip_in_dm("")]
#[embed_success_colour(DARK_BLUE)]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
 ) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
 }

type NikelFunc<T> = fn(Parameters) -> NikelResult<T>;

fn req<T: Clone>(ctx: &Context, msg: &Message, f: NikelFunc<T>, default: &str, proc: fn(T, &mut CreateEmbed)) -> CommandResult {
    
    msg.channel_id.send_message(ctx, |m| {

        let content = &msg.content_safe(&ctx);
        let ddefault = &default.to_owned();

        let params = match to_params(content, ddefault) {
            Ok(p) => p,
            _ => {
                return m.embed(|e: &mut serenity::builder::CreateEmbed| {
                    e.colour((200, 100, 100)).title("Failed").description("Couldn't parse input")
                })
            }
        };

        match f(params) {
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
#[aliases("course", "classes", "class")]
fn courses(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Course>(&ctx, msg, nikel_rs::courses, "code", |c: Course, m: &mut CreateEmbed| {
        let code = c.code.expect("No course code!?");
        let title = format!("{}{}", code,
            match c.name {
                Some(name) => format!(" - {}", name),
                _ => "Course".to_owned()
            }
        );
        m.title(title)
         .field("Campus", c.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Term", c.term.unwrap_or("Unavailable".to_owned()), true)
         .field("UTM Dist. Req.", c.utm_distribution.unwrap_or("Unavailable".to_owned()), true)
         .field("Prereqs", c.prerequisites.unwrap_or("Unavailable".to_owned()), true)
         .field("Coreqs", c.corequisites.unwrap_or("Unavailable".to_owned()), true)
         .field("Exclusions", c.exclusions.unwrap_or("Unavailable".to_owned()), true)
         .field("Description", c.description.unwrap_or("Unavailable".to_owned()), false)
         .url(format!("{}/{}", AC_COURSES, code));
    })
}

#[command]
#[aliases("textbook")]
fn textbooks(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Textbook>(&ctx, msg, nikel_rs::textbooks, "title", |t: Textbook, m: &mut CreateEmbed| {
        m.title(t.title.unwrap_or("Textbook".to_owned()))
         .field("Price", format!("${}", t.price.unwrap_or(-1.0)), true)
         .field("ISBN", t.isbn.unwrap_or("Unavailable".to_owned()), true)
         .field("Courses", t.courses.into_iter().map(|c| c.code.unwrap_or("Unavailable".to_owned()).to_owned()).collect::<Vec<_>>().join("\n"), false);
         match t.image {
            Some(url) => {
                m.image(url);
            },
            _ => {}
         }
         
    })
}

#[command]
#[aliases("exam")]
fn exams(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Exam>(&ctx, msg, nikel_rs::exams, "course_code", |e: Exam, m: &mut CreateEmbed| {
        m.title("Exam")
         .field("Course", e.course_code.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", e.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Date", e.date.unwrap_or("Unavailable".to_owned()), true);
         match e.start {
             Some(time) =>{
                 m.field("Start", util::convert_time(time), true);
             }, _ => {}
         };
         match e.end {
            Some(time) =>{
                m.field("End", util::convert_time(time), true);
            }, _ => {}
        };
    })
}

#[command]
#[aliases("eval")]
fn evals(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Eval>(&ctx, msg, nikel_rs::evals, "name", |e: Eval, m: &mut CreateEmbed| {
        m.title("Eval")
         .field("Name", e.name.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", e.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Last Updated", e.last_updated.unwrap_or("Unavailable".to_owned()), true);
    })
}

#[command]
fn food(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Food>(&ctx, msg, nikel_rs::food, "name", |f: Food, m: &mut CreateEmbed| {
        m.title(f.name.unwrap_or("Food".to_owned()))
         .field("Campus", f.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Address", f.address.unwrap_or("Unavailable".to_owned()), true)
         .field("Tags", f.tags.unwrap_or("Unavailable".to_owned()), true);
         match f.url {
             Some(url) => { m.url(url); },
             _ => {}
         }
         match f.image {
             Some(url) => { m.image(url); },
             _ => {}
         }
    })
}

#[command]
#[aliases("service")]
fn services(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Service>(&ctx, msg, nikel_rs::services, "name", |s: Service, m: &mut CreateEmbed| {
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
#[aliases("building")]
fn buildings(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Building>(&ctx, msg, nikel_rs::buildings, "name", |b: Building, m: &mut CreateEmbed| {
        m.title(b.name.unwrap_or("Building".to_owned()))
         .field("Address", format!("{},{},{}", b.address.street.unwrap_or("?".to_owned()), b.address.city.unwrap_or("?".to_owned()), b.address.country.unwrap_or("?".to_owned())), true)
         .field("Coordinates", format!("{} degrees North, {} degrees East", b.coordinates.latitude.unwrap_or(0.0), b.coordinates.longitude.unwrap_or(0.0)), true);
    })
}

#[command]
fn parking(ctx: &mut Context, msg: &Message) -> CommandResult {
    req::<Parking>(&ctx, msg, nikel_rs::parking, "name", |p: Parking, m: &mut CreateEmbed| {
        m.title("Parking")
         .field("Name", p.name.unwrap_or("Unavailable".to_owned()), true)
         .field("Campus", p.campus.unwrap_or("Unavailable".to_owned()), true)
         .field("Address", p.address.unwrap_or("Unavailable".to_owned()), true)
         .field("Tags", p.description.unwrap_or("Unavailable".to_owned()), false);
    })
}

fn to_params<'a>(input: &'a String, default: &'a String) -> Result<Parameters<'a>, ()> {
    
    let (_, rest) = match split_command(input, ' ') {
        Ok((a, b)) => (a, b),
        _ => return Err(())
    };

    if !rest.contains(":") {
        return Ok(vec![(default, rest)]);
    }
    
    Ok(
        rest.split(',')
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
    )

}

fn split_command(in_string: &str, delim: char) -> Result<(&str, &str), ()> {
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
