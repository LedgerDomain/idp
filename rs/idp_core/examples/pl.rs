#![allow(unused_macros)]

use anyhow::Result;
// use idp_core::PlumRef;
use std::{
    // any::Any,
    boxed::Box,
    collections::HashMap,
    // marker::PhantomData,
    sync::{Arc, RwLock},
};

pub struct Runtime {
    /// Stack of symbol tables, corresponding with the call stack.
    pub symbol_mv: Vec<HashMap<String, Arc<RwLock<ConcreteValue>>>>,
    pub stack_depth_limit: usize,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            symbol_mv: vec![HashMap::new()],
            stack_depth_limit: 0x400,
        }
    }
    pub fn reset(&mut self) {
        self.symbol_mv = vec![HashMap::new()];
    }
    fn push_symbol_table(&mut self) {
        if self.symbol_mv.len() + 1 > self.stack_depth_limit {
            panic!("script programmer error: call stack overflow");
        }
        log::trace!(
            "Runtime::push_symbol_table; before: self.symbol_mv.len(): {}",
            self.symbol_mv.len()
        );
        self.symbol_mv.push(HashMap::new());
        log::trace!(
            "Runtime::push_symbol_table; after: self.symbol_mv.len(): {}",
            self.symbol_mv.len()
        );
    }
    fn pop_symbol_table(&mut self) {
        if self.symbol_mv.len() <= 1 {
            panic!("pl.rs programmer error: call stack underflow");
        }
        log::trace!(
            "Runtime::pop_symbol_table; before: self.symbol_mv.len(): {}",
            self.symbol_mv.len()
        );
        self.symbol_mv.pop();
        log::trace!(
            "Runtime::pop_symbol_table; after: self.symbol_mv.len(): {}",
            self.symbol_mv.len()
        );
    }
    pub fn define(&mut self, symbol_id: String, value: ConcreteValue) -> Result<()> {
        let symbol_m = self.symbol_mv.last_mut().unwrap();
        anyhow::ensure!(
            !symbol_m.contains_key(&symbol_id),
            "symbol {:?} is already defined",
            symbol_id
        );
        log::trace!("defining {:?} to be {:?}", symbol_id, value);
        symbol_m.insert(symbol_id, Arc::new(RwLock::new(value)));
        Ok(())
    }
    pub fn assign(&mut self, symbol_id: &str, value: ConcreteValue) -> Result<()> {
        log::trace!("assigning {:?} to be {:?}", symbol_id, value);
        let symbol_m = self.symbol_mv.last_mut().unwrap();
        let symbol_la = symbol_m
            .get_mut(symbol_id)
            .ok_or_else(|| anyhow::anyhow!("can't assign to undefined symbol {:?}", symbol_id))?;
        *symbol_la.write().unwrap() = value;
        Ok(())
    }
    pub fn dereference(&self, symbol_id: &str) -> Result<Arc<RwLock<ConcreteValue>>> {
        for symbol_m in self.symbol_mv.iter().rev() {
            if let Some(symbol_la) = symbol_m.get(symbol_id) {
                return Ok(symbol_la.clone());
            }
        }
        anyhow::bail!("can't dereference undefined symbol {:?}", symbol_id);
    }
}

/// This handles automatically popping the call stack at the end of its lifetime.
pub struct StackGuard<'a> {
    pub rt: &'a mut Runtime,
}

impl<'a> StackGuard<'a> {
    pub fn new(rt: &'a mut Runtime) -> Self {
        log::trace!("StackGuard::new");
        rt.push_symbol_table();
        Self { rt }
    }
}

impl<'a> Drop for StackGuard<'a> {
    fn drop(&mut self) {
        log::trace!("StackGuard::drop");
        self.rt.pop_symbol_table();
    }
}

pub trait Executable {
    fn exec(&self, rt: &mut Runtime) -> Result<()>;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Statement {
    Definition(Definition),
    Assignment(Assignment),
}

impl Executable for Statement {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        match self {
            Statement::Definition(x) => x.exec(rt),
            Statement::Assignment(x) => x.exec(rt),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Definition {
    pub symbol_id: String,
    pub expr: Expr,
}

impl Executable for Definition {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        rt.define(self.symbol_id.clone(), value)
    }
}

macro_rules! define {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::Definition(Definition {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        })
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Assignment {
    pub symbol_id: String,
    pub expr: Expr,
}

impl Executable for Assignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        rt.assign(self.symbol_id.as_str(), value)
    }
}

macro_rules! assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::Assignment(Assignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        })
    };
}

/// Analogous to Rust's blocks, which are a sequence of zero or more statements, then an expression.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Block {
    pub statement_v: Vec<Statement>,
    pub expr: Expr,
}

impl Block {
    pub fn run_as_program(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        assert_eq!(rt.symbol_mv.len(), 1, "Block::run_as_program can only be used when the Runtime's symbol table stack is depth 1");
        self.eval_impl(rt, false)
    }
    fn eval_impl(&self, rt: &mut Runtime, use_inner_stack_guard: bool) -> Result<ConcreteValue> {
        log::trace!("Block::eval");
        if use_inner_stack_guard {
            let stack_guard = StackGuard::new(rt);
            for statement in self.statement_v.iter() {
                statement.exec(stack_guard.rt)?;
            }
            let value = self.expr.eval(stack_guard.rt)?;
            Ok(value)
        } else {
            for statement in self.statement_v.iter() {
                statement.exec(rt)?;
            }
            self.expr.eval(rt)
        }
    }
}

impl Value for Block {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        self.eval_impl(rt, true)
        // log::trace!("Block::eval");
        // let stack_guard = StackGuard::new(rt);
        // for statement in self.statement_v.iter() {
        //     statement.exec(stack_guard.rt)?;
        // }
        // let value = self.expr.eval(stack_guard.rt)?;
        // Ok(value)
    }
}

/// NOTE/TODO: I couldn't figure out how to get it to parse Rust-style block syntax
/// like `block! { x; y; z; value }`, instead, for now, you have to do
/// `block! { x; y; z;; value }`.
/// TODO: Maybe need to put the more complex rule first, since apparently macro parsing
/// is very simple and can't do any backtracking.
macro_rules! block {
    ($x:expr) => {
        Expr::Block(Box::new(Block {
            statement_v: Vec::new(),
            expr: $x,
        }))
    };
    ($($statement:expr;)+; $x:expr) => {
        Expr::Block(Box::new(Block {
            statement_v: vec![$($statement,)+],
            expr: $x,
        }))
    };
}

#[derive(Clone, Debug, derive_more::From)]
pub enum ConcreteValue {
    Float64(Float64),
    Function(Function),
}

impl ConcreteValue {
    pub fn as_float64(&self) -> Result<&Float64> {
        match self {
            ConcreteValue::Float64(float64) => Ok(float64),
            _ => {
                anyhow::bail!("expected Float64 but got {:?}", self);
            }
        }
    }
    pub fn as_function(&self) -> Result<&Function> {
        match self {
            ConcreteValue::Function(function) => Ok(function),
            _ => {
                anyhow::bail!("expected Function but got {:?}", self);
            }
        }
    }
}

impl From<f64> for ConcreteValue {
    fn from(x: f64) -> Self {
        ConcreteValue::Float64(Float64(x))
    }
}

// impl From<Float64> for ConcreteValue {
//     fn from(float64: Float64) -> Self {
//         ConcreteValue::Float64(float64)
//     }
// }

// impl From<Function> for ConcreteValue {
//     fn from(function: Function) -> Self {
//         ConcreteValue::Function(function)
//     }
// }

// TODO: Rename to Expr after Expr is renamed to ASTNode.
pub trait Value {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue>;
}

// impl<T: Any + std::fmt::Debug + serde::de::DeserializeOwned> Value for idp_core::PlumRef<T>
// where
//     ConcreteValue: TryFrom<T>,
// {
//     fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
//         // The call to value() will handle loading, deserializing, and caching into memory.
//         let value_a = self.value()?;
//         Ok(ConcreteValue::try_from(value_a.into())?)
//     }
// }

impl Value for idp_core::PlumRef<Expr> {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // The call to value() will handle loading, deserializing, and caching into memory.
        self.value()?.eval(rt)
    }
}

macro_rules! plum_ref {
    ($plum_head_seal:expr) => {
        Expr::PlumRef(Box::new(idp_core::PlumRef::<Expr>::new($plum_head_seal)))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Function {
    /// All arguments are implicitly typed as f64 for now.
    pub argument_name_v: Vec<String>,
    // TODO: return type (for now, this is implicitly f64)
    pub body: Expr,
}

impl Value for Function {
    fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        // Inefficient, but fine for now.
        Ok(ConcreteValue::Function(self.clone()))
    }
}

macro_rules! function {
    (($($argument_name_v:ident),*) -> $body:expr) => {
        Expr::Function(Box::new(Function {
            argument_name_v: vec![$(stringify!($argument_name_v).to_string(),)*],
            body: $body,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Call {
    pub function: Expr,
    /// All arguments are implicitly typed as f64 for now.
    pub argument_expr_v: Vec<Expr>,
}

impl Value for Call {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        log::trace!("Call::eval");
        let function_eval = self.function.eval(rt)?;
        let function = function_eval.as_function()?;
        let stack_guard = StackGuard::new(rt);
        for (argument_name, argument_expr) in
            std::iter::zip(function.argument_name_v.iter(), self.argument_expr_v.iter())
        {
            let argument_value = argument_expr.eval(stack_guard.rt)?;

            stack_guard
                .rt
                .define(argument_name.clone(), argument_value)?;
        }
        let retval = function.body.eval(stack_guard.rt)?;
        Ok(retval)
    }
}

macro_rules! call {
    ($function:expr, ($($argument_expr_v:expr),*)) => {
        Expr::Call(Box::new(Call {
            function: $function,
            argument_expr_v: vec![$($argument_expr_v,)*],
        }))
    };
}

#[derive(
    Clone,
    Copy,
    Debug,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::Display,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
    serde::Serialize,
)]
pub struct Float64(f64);

impl Float64 {
    fn as_f64(self) -> f64 {
        self.0
    }
}

impl Value for Float64 {
    fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        Ok(ConcreteValue::Float64(*self))
    }
}

macro_rules! float64 {
    ($x: expr) => {
        Expr::Float64(Float64($x))
    };
}

#[derive(Clone, Debug, derive_more::Deref, serde::Deserialize, serde::Serialize)]
pub struct SymbolicRef(String);

impl Value for SymbolicRef {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // This is inefficient, but is fine for now.  The solution would be to return
        // Arc<RwLock<ConcreteValue>> from Value::eval, but that has its own drawbacks.
        Ok(rt.dereference(self.as_str())?.read().unwrap().clone())
    }
}

macro_rules! symbolic_ref {
    ($symbol_id: ident) => {
        Expr::SymbolicRef(SymbolicRef(stringify!($symbol_id).to_string()))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neg {
    pub operand: Expr,
}

impl Value for Neg {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let x = self.operand.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(-x)))
    }
}

macro_rules! neg {
    ($x: expr) => {
        Expr::Neg(Box::new(Neg($x)))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Add {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Value for Add {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs + rhs)))
    }
}

macro_rules! add {
    ($lhs: expr, $rhs: expr $(,)?) => {
        Expr::Add(Box::new(Add {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Sub {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Value for Sub {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs - rhs)))
    }
}

macro_rules! sub {
    ($lhs: expr, $rhs: expr $(,)?) => {
        Expr::Sub(Box::new(Sub {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Mul {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Value for Mul {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs * rhs)))
    }
}

macro_rules! mul {
    ($lhs: expr, $rhs: expr $(,)?) => {
        Expr::Mul(Box::new(Mul {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Div {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Value for Div {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs / rhs)))
    }
}

macro_rules! div {
    ($lhs: expr, $rhs: expr $(,)?) => {
        Expr::Div(Box::new(Div {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Pow {
    pub base: Expr,
    pub exponent: Expr,
}

impl Value for Pow {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let base = self.base.eval(rt)?.as_float64()?.as_f64();
        let exponent = self.exponent.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(base.powf(exponent))))
    }
}

macro_rules! pow {
    ($base: expr, $exponent: expr $(,)?) => {
        Expr::Pow(Box::new(Pow {
            base: $base,
            exponent: $exponent,
        }))
    };
}

// TODO: This could be called ASTNode
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Expr {
    Float64(Float64),
    SymbolicRef(SymbolicRef),
    Neg(Box<Neg>),
    Add(Box<Add>),
    Sub(Box<Sub>),
    Mul(Box<Mul>),
    Div(Box<Div>),
    Pow(Box<Pow>),
    Block(Box<Block>),
    Function(Box<Function>),
    Call(Box<Call>),
    PlumRef(Box<idp_core::PlumRef<Expr>>),
}

// TEMP HACK maybe?  Seems to be needed for some reason.
impl Default for Expr {
    fn default() -> Self {
        // Arbitrary choice
        Expr::Float64(Float64(0.0))
    }
}

// TEMP HACK
unsafe impl Send for Expr {}
unsafe impl Sync for Expr {}

impl Expr {
    pub fn into_block(self) -> Result<Block> {
        match self {
            Expr::Block(block_b) => Ok(*block_b),
            _ => {
                anyhow::bail!("expected Block");
            }
        }
    }
}

impl Value for Expr {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        match self {
            Expr::Float64(x) => x.eval(rt),
            Expr::Block(x) => x.eval(rt),
            Expr::SymbolicRef(x) => x.eval(rt),
            Expr::Neg(x) => x.eval(rt),
            Expr::Add(x) => x.eval(rt),
            Expr::Sub(x) => x.eval(rt),
            Expr::Mul(x) => x.eval(rt),
            Expr::Div(x) => x.eval(rt),
            Expr::Pow(x) => x.eval(rt),
            Expr::Function(x) => x.eval(rt),
            Expr::Call(x) => x.eval(rt),
            Expr::PlumRef(x) => x.eval(rt),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let mut rt = Runtime::new();

    {
        let expr = float64!(123.456);
        log::debug!("expr -> {:.17}", expr.eval(&mut rt)?.as_float64()?);
    }

    {
        // cos(1) ~= 1 - 1/2*(1 - 1/(3*4)*(1 - 1/(5*6)*(1 - 1/(7*8)*(1 - 1/(9*10)))))
        //         = 0.54030230379188715
        // is correct to 7 digits; actual value is 0.5403023058681398...
        let cos_1 = sub!(
            float64!(1.0),
            mul!(
                float64!(1.0 / 2.0),
                sub!(
                    float64!(1.0),
                    mul!(
                        float64!(1.0 / (3.0 * 4.0)),
                        sub!(
                            float64!(1.0),
                            mul!(
                                float64!(1.0 / (5.0 * 6.0)),
                                sub!(
                                    float64!(1.0),
                                    mul!(
                                        float64!(1.0 / (7.0 * 8.0)),
                                        sub!(float64!(1.0), float64!(1.0 / (9.0 * 10.0)),)
                                    )
                                )
                            )
                        )
                    )
                )
            )
        );
        log::debug!("cos_1 -> {:.17}", cos_1.eval(&mut rt)?.as_float64()?);
    }

    {
        let block_0 = block!(float64!(1.2));
        log::debug!("block_0 -> {:.17}", block_0.eval(&mut rt)?.as_float64()?);
    }

    {
        let block_1 = block! {
            define!(x, float64!(303.404));
            define!(y, float64!(888.9999));;
            symbolic_ref!(x)
        };
        log::debug!("block_1 -> {:.17}", block_1.eval(&mut rt)?.as_float64()?);
    }

    {
        // Compute cos(2)
        rt.reset();
        let cos_2 = block! {
            define!(x, float64!(2.0));
            define!(x_squared, mul!(symbolic_ref!(x), symbolic_ref!(x)));
            define!(z, mul!(float64!(1.0/(1.0*2.0)), symbolic_ref!(x_squared)));
            define!(y, float64!(1.0));
            assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(3.0*4.0)))));
            assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(5.0*6.0)))));
            assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(7.0*8.0)))));
            assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(9.0*10.0)))));
            assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(11.0*12.0)))));
            assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(13.0*14.0)))));
            assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(15.0*16.0)))));
            assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
            assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(17.0*18.0)))));
            ;
            symbolic_ref!(y)
        };
        log::debug!("cos_2 -> {:.17}", cos_2.eval(&mut rt)?.as_float64()?);
    }

    // Define some functions (which will be used in several places)
    let norm_squared_function = function!((x, y) -> block! {
        define!(z, mul!(symbolic_ref!(x), symbolic_ref!(x)));
        assign!(z, add!(symbolic_ref!(z), mul!(symbolic_ref!(y), symbolic_ref!(y))));
        ;
        symbolic_ref!(z)
    });
    let exp_function = function!((x) -> block! {
        define!(z, symbolic_ref!(x));
        define!(y, float64!(1.0));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/2.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/3.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/4.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/5.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/6.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/7.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/8.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/9.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/10.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/11.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/12.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/13.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/14.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/15.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x), float64!(1.0/16.0))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        ;
        symbolic_ref!(y)
    });
    let cos_function = function!((x) -> block! {
        define!(x_squared, mul!(symbolic_ref!(x), symbolic_ref!(x)));
        define!(z, mul!(float64!(1.0/(1.0*2.0)), symbolic_ref!(x_squared)));
        define!(y, float64!(1.0));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(3.0*4.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(5.0*6.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(7.0*8.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(9.0*10.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(11.0*12.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(13.0*14.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(15.0*16.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(17.0*18.0)))));
        ;
        symbolic_ref!(y)
    });
    let sin_function = function!((x) -> block! {
        define!(x_squared, mul!(symbolic_ref!(x), symbolic_ref!(x)));
        define!(z, mul!(float64!(1.0/(1.0*2.0*3.0)), mul!(symbolic_ref!(x_squared), symbolic_ref!(x))));
        define!(y, symbolic_ref!(x));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(4.0*5.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(6.0*7.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(8.0*9.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(10.0*11.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(12.0*13.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(14.0*15.0)))));
        assign!(y, sub!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(16.0*17.0)))));
        assign!(y, add!(symbolic_ref!(y), symbolic_ref!(z)));
        assign!(z, mul!(symbolic_ref!(z), mul!(symbolic_ref!(x_squared), float64!(1.0/(18.0*19.0)))));
        ;
        symbolic_ref!(y)
    });

    {
        log::debug!("------------------------------------------------------");
        rt.reset();
        let program = block! {
            define!(norm_squared, norm_squared_function.clone());
            define!(exp, exp_function.clone());
            define!(cos, cos_function.clone());
            define!(sin, sin_function.clone());
            ;
            // call!(symbolic_ref!(norm_squared), (call!(symbolic_ref!(cos), (float64!(0.83))), call!(symbolic_ref!(sin), (float64!(0.83)))))
            call!(symbolic_ref!(exp), (float64!(1.0)))
        }
        .into_block()?;
        log::debug!(
            "program -> {:.17}",
            program.run_as_program(&mut rt)?.as_float64()?
        );
        log::debug!("rt.symbol_mv.len(): {}", rt.symbol_mv.len());
        log::debug!(
            "rt.symbol_mv.last().unwrap().keys():\n{:#?}",
            rt.symbol_mv.last().unwrap().keys()
        );
    }

    // Now create a Datahost and Datacache.
    let datahost_la = Arc::new(RwLock::new(idp_core::Datahost::open_in_memory()?));
    // let datacache_la = Arc::new(RwLock::new(idp_core::Datacache::new(datahost_la.clone())));

    idp_core::initialize_datacache(idp_core::Datacache::new(datahost_la.clone()));

    // Store the functions in the Datahost
    let norm_squared_plum_head_seal = datahost_la.write().unwrap().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&norm_squared_function)?)
            .build()?,
    )?;
    let exp_plum_head_seal = datahost_la.write().unwrap().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&exp_function)?)
            .build()?,
    )?;
    let cos_plum_head_seal = datahost_la.write().unwrap().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&cos_function)?)
            .build()?,
    )?;
    let sin_plum_head_seal = datahost_la.write().unwrap().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&sin_function)?)
            .build()?,
    )?;

    {
        log::debug!("------------------------------------------------------");
        rt.reset();
        let program = block! {
            define!(norm_squared, plum_ref!(norm_squared_plum_head_seal.clone()));
            define!(exp, plum_ref!(exp_plum_head_seal.clone()));
            define!(cos, plum_ref!(cos_plum_head_seal.clone()));
            define!(sin, plum_ref!(sin_plum_head_seal.clone()));
            ;
            // call!(symbolic_ref!(norm_squared), (call!(symbolic_ref!(cos), (float64!(0.83))), call!(symbolic_ref!(sin), (float64!(0.83)))))
            call!(symbolic_ref!(exp), (float64!(1.0)))
        }
        .into_block()?;
        log::debug!(
            "program with PlumRef -> {:.17}",
            program.run_as_program(&mut rt)?.as_float64()?
        );
        log::debug!("rt.symbol_mv.len(): {}", rt.symbol_mv.len());
        log::debug!(
            "rt.symbol_mv.last().unwrap().keys():\n{:#?}",
            rt.symbol_mv.last().unwrap().keys()
        );
    }

    Ok(())
}
