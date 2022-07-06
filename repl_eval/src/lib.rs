use roc_parse::ast::Expr;
use roc_std::RocDec;

pub mod eval;
pub mod gen;

pub trait ReplApp<'a> {
    type Memory: 'a + ReplAppMemory;

    /// Run user code that returns a type with a `Builtin` layout
    /// Size of the return value is statically determined from its Rust type
    /// The `transform` callback takes the app's memory and the returned value
    fn call_function<Return, F>(&self, main_fn_name: &str, transform: F) -> Expr<'a>
    where
        F: Fn(&'a Self::Memory, Return) -> Expr<'a>,
        Self::Memory: 'a;

    /// Run user code that returns a struct or union, whose size is provided as an argument
    /// The `transform` callback takes the app's memory and the address of the returned value
    fn call_function_dynamic_size<T, F>(
        &self,
        main_fn_name: &str,
        ret_bytes: usize,
        transform: F,
    ) -> T
    where
        F: Fn(&'a Self::Memory, usize) -> T,
        Self::Memory: 'a;
}

pub trait ReplAppMemory {
    fn deref_bool(&self, addr: usize) -> bool;

    fn deref_u8(&self, addr: usize) -> u8;
    fn deref_u16(&self, addr: usize) -> u16;
    fn deref_u32(&self, addr: usize) -> u32;
    fn deref_u64(&self, addr: usize) -> u64;
    fn deref_u128(&self, addr: usize) -> u128;
    fn deref_usize(&self, addr: usize) -> usize;

    fn deref_i8(&self, addr: usize) -> i8;
    fn deref_i16(&self, addr: usize) -> i16;
    fn deref_i32(&self, addr: usize) -> i32;
    fn deref_i64(&self, addr: usize) -> i64;
    fn deref_i128(&self, addr: usize) -> i128;
    fn deref_isize(&self, addr: usize) -> isize;

    fn deref_f32(&self, addr: usize) -> f32;
    fn deref_f64(&self, addr: usize) -> f64;

    fn deref_dec(&self, addr: usize) -> RocDec {
        let bits = self.deref_i128(addr);
        RocDec::new(bits)
    }

    fn deref_str(&self, addr: usize) -> &str;
}
