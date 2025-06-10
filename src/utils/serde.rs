pub fn is_false(value: &bool) -> bool {
    !value
}

pub fn is_true(value: &bool) -> bool {
    *value
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}