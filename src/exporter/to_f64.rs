pub trait ToF64 {
    fn to_f64(self) -> f64;
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for u64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for i64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

pub trait ToI64 {
    fn to_i64(self) -> i64;
}

impl ToI64 for i64 {
    fn to_i64(self) -> i64 {
        self
    }
}

impl ToI64 for u64 {
    fn to_i64(self) -> i64 {
        match i64::try_from(self) {
            Ok(v) => v,
            Err(_) => i64::MAX,
        }
    }
}
