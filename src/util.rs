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
