use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};

pub fn remove_from_vec<T: PartialEq>(vec: &mut Vec<T>, instance: &T) {
    remove_from_vec_if(vec, &|x: &T| x.eq(instance))
}

pub fn remove_from_vec_if<T>(vec: &mut Vec<T>, predicate: &dyn Fn(&T) -> bool) {
    loop {
        let mut jump = true;
        for obj in vec.iter().enumerate() {
            if predicate(obj.1) {
                vec.remove(obj.0);
                jump = false;
                break;
            }
        }

        if jump {
            break;
        }
    }
}

pub fn vec_any_match<T>(vec: &Vec<T>, predicate: &dyn Fn(&T) -> bool) -> bool {
    for obj in vec {
        if predicate(obj) {
            return true;
        }
    }
    false
}

pub fn vec_all_match<T>(vec: &Vec<T>, predicate: &dyn Fn(&T) -> bool) -> bool {
    for obj in vec {
        if !predicate(obj) {
            return false;
        }
    }
    true
}

pub fn vec_none_match<T>(vec: &Vec<T>, predicate: &dyn Fn(&T) -> bool) -> bool {
    for obj in vec {
        if predicate(obj) {
            return false;
        }
    }
    true
}

pub fn get_month_str(month: u32) -> String {
    match month {
        1 => String::from("Jan"),
        2 => String::from("Feb"),
        3 => String::from("Mar"),
        4 => String::from("Apr"),
        5 => String::from("May"),
        6 => String::from("Jun"),
        7 => String::from("Jul"),
        8 => String::from("Aug"),
        9 => String::from("Sep"),
        10 => String::from("Oct"),
        11 => String::from("Nov"),
        12 => String::from("Dec"),
        _ => unreachable!(),
    }
}

pub fn parse_date_and_time(string: &str) -> Option<NaiveDateTime> {
    let temp_str = string.replace('/', "-");
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

pub fn parse_date(string: &str) -> Option<NaiveDate> {
    let temp_str = string.replace('/', "-");
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

pub fn join_str_with(vec: &Vec<String>, with: &str) -> String {
    let mut string = String::new();
    for str_temp in vec {
        string = format!("{string}{with}{str_temp}")
    }
    string
}
