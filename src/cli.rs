use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use clap::{arg, ArgMatches, Command};

use crate::base::{self, Todo, TodoInstance};

pub fn execute() {
    let mut instance = TodoInstance::create(".");
    instance.read_all();
    match cli().get_matches().subcommand() {
        Some(("init", _)) => {
            base::init_repo(".");
        }
        Some(("create", matches)) => {
            let mut todo = Todo::create(matches.get_one::<String>("TITLE").unwrap().to_owned());
            process_edit_todo(matches, &mut todo);
            instance.todos.push(todo);
        }
        _ => unreachable!(),
    };
    instance.write_all();
}

fn cli() -> Command {
    Command::new("todo")
        .about("A simple to-do manager")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("init").about("Initialize a new todo repo"))
        .subcommand(
            Command::new("create")
                .about("Create a new todo into the repo")
                .arg(arg!(<TITLE> "The name of the todo"))
                .args(edit_args()),
        )
}

fn edit_args() -> Vec<clap::Arg> {
    vec![
        arg!(-n --name <NAME>).required(false),
        arg!(-d --details <DETAILS>).required(false),
        arg!(-w --date <DATE>).required(false),
        arg!(--ddl <DEADLINE>).required(false),
        arg!(--weight <WEIGHT>).required(false),
    ]
}

fn process_edit_todo(matches: &ArgMatches, todo: &mut Todo) {
    if let Some(n) = matches.get_one::<String>("name") {
        todo.metadata.name = n.to_owned()
    }

    if let Some(n) = matches.get_one::<String>("details") {
        todo.metadata.details = n.to_owned()
    }

    if let Some(n) = matches.get_one::<u32>("weight") {
        todo.weight = *n
    }

    if let Some(n) = matches.get_one::<String>("ddl") {
        if let Some(t) = parse_date_and_time(n.to_owned()) {
            todo.deadline = Option::Some(t);
        }
    }
}

fn parse_date_and_time(string: String) -> Option<NaiveDateTime> {
    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%Y/%m/%d-%H:%M:%S") {
        return Option::Some(r);
    }
    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%m/%d-%H:%M:%S") {
        return Option::Some(r);
    }

    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%Y/%m/%d-%H:%M") {
        return Option::Some(r);
    }

    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%Y/%m/%d") {
        return Option::Some(r);
    }

    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%m/%d") {
        return Option::Some(r);
    }

    if let Ok(r) = NaiveDateTime::parse_from_str(&string, "%m/%d-%H:%M") {
        return Option::Some(r);
    }
    Option::None
}
