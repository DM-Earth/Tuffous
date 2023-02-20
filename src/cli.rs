use chrono::{Local, NaiveDate, NaiveDateTime};
use clap::{arg, Arg, ArgMatches, Command};

use crate::base::{self, Todo, TodoInstance};

pub fn execute() {
    let mut instance = TodoInstance::create(".");
    instance.read_all();
    match cli().get_matches().subcommand() {
        Some(("init", _)) => {
            base::init_repo(".");
        }
        Some(("new", matches)) => {
            let mut todo = Todo::create(matches.get_one::<String>("TITLE").unwrap().to_owned());
            process_edit_todo(matches, &mut todo);
            instance.todos.push(todo);
            instance.write_all();
        }
        _ => unreachable!(),
    }
}

fn cli() -> Command {
    Command::new("todo")
        .about("A simple to-do manager")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("init").about("Initialize a new todo repo"))
        .subcommand(
            Command::new("new")
                .about("Create a new todo into the repo")
                .arg(arg!(<TITLE> "The name of the todo"))
                .args(edit_args()),
        )
        .subcommand(
            Command::new("list")
                .about("List todos with filter")
                .args(filter_args()),
        )
}

fn edit_args() -> Vec<Arg> {
    vec![
        arg!(-n - -name <NAME> "Change name of the target").required(false),
        arg!(-d - -details <DETAILS> "Change details of the target").required(false),
        arg!(-w - -date <DATE> "Change date of the target").required(false),
        arg!(--ddl <DEADLINE> "Change deadline of the target").required(false),
        arg!(--weight <WEIGHT> "Change weight of the target").required(false),
        arg!(-t --tag <TAGS>... "Bind/unbind tags for the target").required(false),
    ]
}

fn filter_args() -> Vec<Arg> {
    vec![
        arg!(--ftoday "Filter with today only todos").required(false),
        arg!(--fdate <DATE> "Filter with date-only todos").required(false),
        arg!(--fdater <DATE_RANGE> "Filter with ranged date-only todos")
            .required(false)
            .num_args(2),
        arg!(--fddl <DDL> "Filter with ddl-only todos").required(false),
        arg!(--fddlr <DDL_RANGE> "Filter with ranged ddl-only todos")
            .required(false)
            .num_args(2),
        arg!(--flogged "Filter with logged todos").required(false),
        arg!(--ftag <TAGS>... "Filter with tags").required(false),
        arg!(--fname <NAME> "Search with name").required(false),
    ]
}

fn process_edit_todo(matches: &ArgMatches, todo: &mut Todo) {
    if let Some(n) = matches.get_one::<String>("name") {
        todo.metadata.name = n.to_owned();
    }

    if let Some(n) = matches.get_one::<String>("details") {
        todo.metadata.details = n.to_owned();
    }

    if let Some(n) = matches.get_one::<u32>("weight") {
        todo.weight = *n;
    }

    if let Some(n) = matches.get_one::<String>("ddl") {
        if let Some(t) = parse_date_and_time(n) {
            todo.deadline = Option::Some(t);
        }
    }

    if let Some(n) = matches.get_one::<String>("date") {
        if let Some(d) = parse_date(n) {
            todo.time = Option::Some(d);
        }
    }

    if let Some(ns) = matches.get_many::<String>("tag") {
        for n in ns {
            if let Some(x) = n.strip_prefix("!") {
                let xs = x.to_owned();
                if todo.tags.contains(&xs) {
                    for t in todo.tags.iter().enumerate() {
                        if t.1.eq(&xs) {
                            todo.tags.remove(t.0);
                            break;
                        }
                    }
                }
            } else {
                let ns = n.to_owned();
                if !todo.tags.contains(&ns) {
                    todo.tags.push(ns);
                }
            }
        }
    }
}

fn parse_date_and_time(string: &String) -> Option<NaiveDateTime> {
    let fmts = vec![
        "%Y/%m/%d-%H:%M:%S",
        "%m/%d-%H:%M:%S",
        "%Y/%m/%d-%H:%M",
        "%Y/%m/%d",
        "%m/%d",
        "%m/%d-%H:%M",
    ];

    for fmt in fmts {
        if let Ok(r) = NaiveDateTime::parse_from_str(string, fmt) {
            return Option::Some(r);
        }
    }

    Option::None
}

fn parse_date(string: &String) -> Option<NaiveDate> {
    let fmts = vec!["%Y/%m/%d", "%m/%d"];

    for fmt in fmts {
        if let Ok(r) = NaiveDate::parse_from_str(string, fmt) {
            return Option::Some(r);
        }
    }

    if string.to_lowercase().contains("today") {
        return Option::Some(Local::now().date_naive());
    }

    Option::None
}

struct TodoScanner {
    pub instance: TodoInstance,
    pub cache: Vec<u64>,
}

impl TodoScanner {
    pub fn new(instance: TodoInstance) -> Self {
        TodoScanner {
            instance,
            cache: Vec::new(),
        }
    }

    pub fn apply_filters(&mut self, matches: &ArgMatches) {
        self.cache.clear();
        for todo_id in self.instance.get_todos() {
            if !self.cache.contains(&todo_id)
                && Self::match_filters(matches, self.instance.get(&todo_id).unwrap(), true)
            {
                self.cache.push(todo_id);
                for father_todo_id in self.instance.get_all_deps(&todo_id) {
                    if !self.cache.contains(&father_todo_id) {
                        self.cache.push(father_todo_id);
                    }
                }

                for child_todo_id in self.instance.get_children(&todo_id) {
                    if !self.cache.contains(&child_todo_id)
                        && Self::match_filters(matches, self.instance.get(&todo_id).unwrap(), false)
                    {
                        self.cache.push(child_todo_id);
                    }
                }
            }
        }
    }

    fn match_filters(matches: &ArgMatches, todo: &Todo, strict: bool) -> bool {
        if matches.contains_id("flogged") {
            if !todo.completed {
                return false;
            }
        } else if todo.completed {
            return false;
        }

        if matches.contains_id("ftoday") {
            if let Some(d) = todo.time {
                if !d.eq(&Local::now().date_naive()) {
                    return false;
                }
            }
        }

        if let Some(n) = matches.get_one::<String>("fdate") {
            if let Some(d) = todo.time {
                if let Some(m) = parse_date(n) {
                    if !d.eq(&m) {
                        return false;
                    }
                }
            }
        }

        if let Some(n) = matches.get_many::<String>("fdater") {
            if let Some(date) = todo.time {
                let mut index = 0;
                let mut skip = false;
                for nd in n {
                    index += 1;
                    if index == 1 {
                        if let Some(d) = parse_date(nd) {
                            if d > date {
                                skip = true;
                            }
                        }
                    }

                    if index == 2 {
                        if let Some(d) = parse_date(nd) {
                            if d < date {
                                skip = true;
                            }
                        }
                    }
                }

                if skip {
                    return false;
                }
            }
        }

        if strict {
            if let Some(n) = matches.get_one::<String>("fddl") {
                if let Some(d) = todo.deadline {
                    if let Some(m) = parse_date(n) {
                        if !d.date().eq(&m) {
                            return false;
                        }
                    }
                }
            }

            if let Some(n) = matches.get_many::<String>("fddlr") {
                if let Some(ddl) = todo.deadline {
                    let mut index = 0;
                    let mut skip = false;
                    let date = ddl.date();
                    for nd in n {
                        index += 1;
                        if index == 1 {
                            if let Some(d) = parse_date(nd) {
                                if d > date {
                                    skip = true;
                                }
                            }
                        }

                        if index == 2 {
                            if let Some(d) = parse_date(nd) {
                                if d < date {
                                    skip = true;
                                }
                            }
                        }
                    }

                    if skip {
                        return false;
                    }
                }
            }

            if let Some(n) = matches.get_many::<String>("ftag") {
                let mut skip = false;
                for nd in n {
                    if let Some(np) = nd.strip_prefix("!") {
                        if todo.tags.contains(&np.to_string()) {
                            skip = true;
                            break;
                        }
                    } else if !todo.tags.contains(nd) {
                        skip = true;
                        break;
                    }
                }

                if skip {
                    return false;
                }
            }

            if let Some(n) = matches.get_one::<String>("fname") {
                if !todo
                    .metadata
                    .name
                    .to_lowercase()
                    .contains(&n.to_lowercase())
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn list_and_choose(&self) -> Vec<u64> {
        let mut temp = self.cache.clone();

        while !temp.is_empty() {
            for todo_id in &self.cache {
                let todo = self.instance.get(todo_id).unwrap();
                let mut has_dep = false;
                for dep in &todo.dependents {
                    if temp.contains(dep) {
                        has_dep = true;
                        break;
                    }
                }

                if !has_dep {}
            }
        }

        vec![]
    }
}
