use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike};
use clap::{arg, Arg, ArgMatches, Command};

use crate::base::{self, Todo, TodoInstance};

pub fn execute() {
    match cli().get_matches().subcommand() {
        Some(("init", _)) => {
            base::init_repo(".");
        }
        Some(("new", matches)) => {
            let mut instance = TodoInstance::create(".");
            instance.read_all();
            let mut todo = Todo::create(matches.get_one::<String>("TITLE").unwrap().to_owned());
            process_edit_todo(matches, &mut todo);
            instance.todos.push(todo);
            instance.write_all();
        }
        Some(("list", matches)) => {
            let mut scanner = TodoScanner::new(TodoInstance::create("."));
            scanner.instance.read_all();
            scanner.apply_filters(matches);
            scanner.list(false);
        }
        Some(("edit", matches)) => {
            let mut scanner = TodoScanner::new(TodoInstance::create("."));
            scanner.instance.read_all();
            scanner.apply_filters(matches);
            for todo_id in scanner.list(true) {
                if let Some(todo) = scanner.instance.get_mut(&todo_id) {
                    process_edit_todo(matches, todo);
                }
            }
            scanner.instance.write_all();
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
        .subcommand(
            Command::new("edit")
                .about("Edit todos with filter")
                .args(filter_args())
                .args(edit_args()),
        )
}

fn edit_args() -> Vec<Arg> {
    vec![
        arg!(-n --name <NAME> "Change name of the target").required(false),
        arg!(-d --details <DETAILS> "Change details of the target").required(false),
        arg!(-w --date <DATE> "Change date of the target").required(false),
        arg!(--ddl <DEADLINE> "Change deadline of the target").required(false),
        arg!(--weight <WEIGHT> "Change weight of the target").required(false),
        arg!(-t --tag <TAGS>... "Bind/unbind tags for the target").required(false),
    ]
}

fn filter_args() -> Vec<Arg> {
    vec![
        arg!(--ftoday <TODAY> "Filter with today only todos").default_value("false"),
        arg!(--fdate <DATE> "Filter with date-only todos").required(false),
        arg!(--fdater <DATE_RANGE> "Filter with ranged date-only todos")
            .required(false)
            .num_args(2),
        arg!(--fddl <DDL> "Filter with ddl-only todos").required(false),
        arg!(--fddlr <DDL_RANGE> "Filter with ranged ddl-only todos")
            .required(false)
            .num_args(2),
        arg!(--flogged <LOGGED> "Filter with logged todos").default_value("false"),
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
    let temp_str = string.replace("/", "-");
    let now = Local::now();

    for variant in vec![
        format!("{}-{}", now.year(), temp_str),
        format!("{}-{}-00:00:00", now.year(), temp_str),
        format!("{}-{}:00", now.year(), temp_str),
        format!("{}", temp_str),
        format!("{}-00:00:00", temp_str),
        format!("{}:00", temp_str),
    ] {
        if let Ok(r) = NaiveDateTime::parse_from_str(&variant, "%Y-%m-%d-%H:%M:%S") {
            return Option::Some(r);
        }
    }

    if string.to_lowercase().contains("now") {
        return Option::Some(Local::now().naive_local());
    }

    Option::None
}

fn parse_date(string: &String) -> Option<NaiveDate> {
    let temp_str = string.replace("/", "-");
    let now = Local::now();

    for variant in vec![
        format!("{}-{}", now.year(), temp_str),
        format!("{}", temp_str),
    ] {
        if let Ok(r) = NaiveDate::parse_from_str(&variant, "%Y-%m-%d") {
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
        if matches.get_one::<String>("flogged").unwrap().eq("true") {
            if !todo.completed {
                return false;
            }
        } else if todo.completed {
            return false;
        }

        if let Some(n) = matches.get_one::<String>("ftoday") {
            if n.eq("true") {
                if let Some(d) = todo.time {
                    if !date_eq(&d, &Local::now().date_naive()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        if let Some(n) = matches.get_one::<String>("fdate") {
            if let Some(d) = todo.time {
                if let Some(m) = parse_date(n) {
                    if !date_eq(&d, &m) {
                        return false;
                    }
                }
            } else {
                return false;
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
            } else {
                return false;
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
                } else {
                    return false;
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
                } else {
                    return false;
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

    pub fn list(&self, choose: bool) -> Vec<u64> {
        let mut vec = Vec::new();

        for todo_id in &self.cache {
            let todo = self.instance.get(todo_id).unwrap();
            let mut has_dep = false;
            for dep in &todo.dependents {
                if self.cache.contains(dep) {
                    has_dep = true;
                    break;
                }
            }

            if !has_dep {
                vec.append(&mut self.as_tree(todo_id, &self.cache));
            }
        }

        if vec.is_empty() {
            Vec::new()
        } else {
            println!("{} todos:", vec.len());
            if choose {
                for todo in vec.iter().enumerate() {
                    println!("[{}] {}", todo.0 + 1, todo.1.string);
                }

                println!("\nPlease enter your selection:");

                let mut ret_vec = Vec::new();

                for sel in parse_selection(&input_string()) {
                    for v in vec.iter().enumerate() {
                        if v.0 == sel as usize - 1 {
                            ret_vec.push(v.1.id);
                        }
                    }
                }

                ret_vec
            } else {
                for todo in &vec {
                    println!("{}", todo.string);
                }
                vec![]
            }
        }
    }

    fn as_tree(&self, id: &u64, range: &Vec<u64>) -> Vec<FormattedTodo> {
        let todo = self.instance.get(&id).unwrap();
        let mut vec = Vec::new();
        vec.push(FormattedTodo::of(*id, format_todo(todo)));
        for child in self.instance.get_children_once(id) {
            if range.contains(&child) {
                vec.append(&mut self.as_tree(&child, range));
            }
        }

        vec
    }
}

fn format_todo(todo: &Todo) -> String {
    format!(
        "└─ {}{} {}{}",
        if todo.metadata.details.is_empty() {
            todo.metadata.name.to_owned()
        } else {
            format!("{}: {}", todo.metadata.name, todo.metadata.details)
        },
        {
            let mut temp = String::new();
            for tag in &todo.tags {
                temp = format!("{} [{}]", temp, tag);
            }
            temp
        },
        {
            if let Some(date) = todo.time {
                format!(" / DATE- {}", date)
            } else {
                String::new()
            }
        },
        {
            if let Some(ddl) = todo.deadline {
                let real = ddl.and_local_timezone(Local).unwrap();
                format!(" / DDL- {} {}:{}", ddl.date(), real.hour(), real.minute())
            } else {
                String::new()
            }
        }
    )
}

fn date_eq(date1: &NaiveDate, date2: &NaiveDate) -> bool {
    date1.year() == date2.year() && date1.month() == date2.month() && date1.day() == date2.day()
}

fn parse_selection(string: &String) -> Vec<u64> {
    let mut vec = Vec::new();
    for obj in string.replace(",", " ").split_whitespace() {
        if let Ok(num) = obj.parse::<u64>() {
            if !vec.contains(&num) {
                vec.push(num);
            }
        } else if obj.contains("-") {
            let mut n1 = 0;
            let mut n2 = 0;

            {
                let mut index = 0;
                for obj2 in obj.replace("-", " ").split_whitespace() {
                    if let Ok(num2) = obj2.parse::<u64>() {
                        if index == 0 {
                            n1 = num2;
                        }

                        if index == 1 {
                            n2 = num2;
                        }

                        index += 1;
                    }
                }
            }

            if n1 > 0 && n2 > 0 {
                while n1 <= n2 {
                    vec.push(n1);
                    n1 += 1;
                }
            }
        }
    }

    vec
}

fn input_string() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("read_line error!");
    input.lines().next().unwrap().to_string()
}

struct FormattedTodo {
    pub string: String,
    pub id: u64,
}

impl FormattedTodo {
    pub fn of(id: u64, string: String) -> Self {
        FormattedTodo { string, id }
    }
}
