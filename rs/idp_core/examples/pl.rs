#![allow(unused_macros)]

use anyhow::Result;
use parking_lot::RwLock;
use std::{boxed::Box, collections::HashMap, sync::Arc};

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
    // TODO: symbol declaration, deletion

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
    Definition(Box<Definition>),
    Assignment(Box<Assignment>),
    AddAssignment(Box<AddAssignment>),
    SubAssignment(Box<SubAssignment>),
    MulAssignment(Box<MulAssignment>),
    DivAssignment(Box<DivAssignment>),
}

impl Executable for Statement {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        match self {
            Statement::Definition(x) => x.exec(rt),
            Statement::Assignment(x) => x.exec(rt),
            Statement::AddAssignment(x) => x.exec(rt),
            Statement::SubAssignment(x) => x.exec(rt),
            Statement::MulAssignment(x) => x.exec(rt),
            Statement::DivAssignment(x) => x.exec(rt),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Definition {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for Definition {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        rt.define(self.symbol_id.clone(), value)
    }
}

macro_rules! define {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::Definition(Box::new(Definition {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Assignment {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for Assignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write() = value;
        Ok(())
    }
}

macro_rules! assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::Assignment(Box::new(Assignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddAssignment {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for AddAssignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write() += value;
        Ok(())
    }
}

macro_rules! add_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::AddAssignment(Box::new(AddAssignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SubAssignment {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for SubAssignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write() -= value;
        Ok(())
    }
}

macro_rules! sub_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::SubAssignment(Box::new(SubAssignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MulAssignment {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for MulAssignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write() *= value;
        Ok(())
    }
}

macro_rules! mul_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::MulAssignment(Box::new(MulAssignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DivAssignment {
    pub symbol_id: String,
    pub expr: ASTNode,
}

impl Executable for DivAssignment {
    fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt)?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write() /= value;
        Ok(())
    }
}

macro_rules! div_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        Statement::DivAssignment(Box::new(DivAssignment {
            symbol_id: stringify!($symbol_id).to_string(),
            expr: $expr,
        }))
    };
}

/// Analogous to Rust's blocks, which are a sequence of zero or more statements, then an expression.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Block {
    pub statement_v: Vec<Statement>,
    pub expr: ASTNode,
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

impl Expr for Block {
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
        ASTNode::Block(Box::new(Block {
            statement_v: Vec::new(),
            expr: $x,
        }))
    };
    ($($statement:expr;)+; $x:expr) => {
        ASTNode::Block(Box::new(Block {
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
    pub fn as_float64_mut(&mut self) -> Result<&mut Float64> {
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

impl std::ops::AddAssign for ConcreteValue {
    fn add_assign(&mut self, rhs: Self) {
        // This will panic if there's a scripting error, which is obviously not great.
        // the solution would be some sort of TryAddAssign, but that's not actually a
        // Rust trait.
        use std::ops::{Deref, DerefMut};
        *self.as_float64_mut().unwrap().deref_mut() += *rhs.as_float64().unwrap().deref();
    }
}

impl std::ops::SubAssign for ConcreteValue {
    fn sub_assign(&mut self, rhs: Self) {
        // This will panic if there's a scripting error, which is obviously not great.
        // the solution would be some sort of TryAddAssign, but that's not actually a
        // Rust trait.
        use std::ops::{Deref, DerefMut};
        *self.as_float64_mut().unwrap().deref_mut() -= *rhs.as_float64().unwrap().deref();
    }
}

impl std::ops::MulAssign for ConcreteValue {
    fn mul_assign(&mut self, rhs: Self) {
        // This will panic if there's a scripting error, which is obviously not great.
        // the solution would be some sort of TryAddAssign, but that's not actually a
        // Rust trait.
        use std::ops::{Deref, DerefMut};
        *self.as_float64_mut().unwrap().deref_mut() *= *rhs.as_float64().unwrap().deref();
    }
}

impl std::ops::DivAssign for ConcreteValue {
    fn div_assign(&mut self, rhs: Self) {
        // This will panic if there's a scripting error, which is obviously not great.
        // the solution would be some sort of TryAddAssign, but that's not actually a
        // Rust trait.
        use std::ops::{Deref, DerefMut};
        *self.as_float64_mut().unwrap().deref_mut() /= *rhs.as_float64().unwrap().deref();
    }
}

pub trait Expr {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue>;
}

impl Expr for idp_core::PlumRef<ASTNode> {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // The call to value() will handle loading, deserializing, and caching into memory.
        self.value()?.eval(rt)
    }
}

macro_rules! plum_ref {
    ($plum_head_seal:expr) => {
        ASTNode::PlumRef(Box::new(idp_core::PlumRef::<ASTNode>::new($plum_head_seal)))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Function {
    /// All arguments are implicitly typed as f64 for now.
    pub argument_name_v: Vec<String>,
    // TODO: return type (for now, this is implicitly f64)
    pub body: ASTNode,
}

impl Expr for Function {
    fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        // Inefficient, but fine for now.
        Ok(ConcreteValue::Function(self.clone()))
    }
}

macro_rules! function {
    (($($argument_name_v:ident),*) -> $body:expr) => {
        ASTNode::Function(Box::new(Function {
            argument_name_v: vec![$(stringify!($argument_name_v).to_string(),)*],
            body: $body,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Call {
    pub function: ASTNode,
    /// All arguments are implicitly typed as f64 for now.
    pub argument_expr_v: Vec<ASTNode>,
}

impl Expr for Call {
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
        ASTNode::Call(Box::new(Call {
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

impl Expr for Float64 {
    fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        Ok(ConcreteValue::Float64(*self))
    }
}

macro_rules! float64 {
    ($x: expr) => {
        ASTNode::Float64(Float64($x))
    };
}

#[derive(Clone, Debug, derive_more::Deref, serde::Deserialize, serde::Serialize)]
pub struct SymbolicRef(String);

impl Expr for SymbolicRef {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // This is inefficient, but is fine for now.  The solution would be to return
        // Arc<RwLock<ConcreteValue>> from Expr::eval, but that has its own drawbacks.
        Ok(rt.dereference(self.as_str())?.read().clone())
    }
}

macro_rules! symbolic_ref {
    ($symbol_id: ident) => {
        ASTNode::SymbolicRef(SymbolicRef(stringify!($symbol_id).to_string()))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neg {
    pub operand: ASTNode,
}

impl Expr for Neg {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let x = self.operand.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(-x)))
    }
}

macro_rules! neg {
    ($x: expr) => {
        ASTNode::Neg(Box::new(Neg($x)))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Add {
    pub lhs: ASTNode,
    pub rhs: ASTNode,
}

impl Expr for Add {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs + rhs)))
    }
}

macro_rules! add {
    ($lhs: expr, $rhs: expr $(,)?) => {
        ASTNode::Add(Box::new(Add {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Sub {
    pub lhs: ASTNode,
    pub rhs: ASTNode,
}

impl Expr for Sub {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs - rhs)))
    }
}

macro_rules! sub {
    ($lhs: expr, $rhs: expr $(,)?) => {
        ASTNode::Sub(Box::new(Sub {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Mul {
    pub lhs: ASTNode,
    pub rhs: ASTNode,
}

impl Expr for Mul {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs * rhs)))
    }
}

macro_rules! mul {
    ($lhs: expr, $rhs: expr $(,)?) => {
        ASTNode::Mul(Box::new(Mul {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Div {
    pub lhs: ASTNode,
    pub rhs: ASTNode,
}

impl Expr for Div {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt)?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs / rhs)))
    }
}

macro_rules! div {
    ($lhs: expr, $rhs: expr $(,)?) => {
        ASTNode::Div(Box::new(Div {
            lhs: $lhs,
            rhs: $rhs,
        }))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Pow {
    pub base: ASTNode,
    pub exponent: ASTNode,
}

impl Expr for Pow {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let base = self.base.eval(rt)?.as_float64()?.as_f64();
        let exponent = self.exponent.eval(rt)?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(base.powf(exponent))))
    }
}

macro_rules! pow {
    ($base: expr, $exponent: expr $(,)?) => {
        ASTNode::Pow(Box::new(Pow {
            base: $base,
            exponent: $exponent,
        }))
    };
}

// TODO: This could be called ASTNode
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum ASTNode {
    Float64(Float64),
    SymbolicRef(SymbolicRef),
    Neg(Box<Neg>),
    Add(Box<Add>),
    Sub(Box<Sub>),
    Mul(Box<Mul>),
    Div(Box<Div>),
    Pow(Box<Pow>),
    // ParenExpr(Box<ParenExpr>),
    Block(Box<Block>),
    Function(Box<Function>),
    Call(Box<Call>),
    PlumRef(Box<idp_core::PlumRef<ASTNode>>),
}

// TEMP HACK
unsafe impl Send for ASTNode {}
unsafe impl Sync for ASTNode {}

impl Default for ASTNode {
    fn default() -> Self {
        // Arbitrary choice
        ASTNode::Float64(Float64(0.0))
    }
}

impl ASTNode {
    pub fn into_block(self) -> Result<Block> {
        match self {
            ASTNode::Block(block_b) => Ok(*block_b),
            _ => {
                anyhow::bail!("expected Block");
            }
        }
    }
}

impl Expr for ASTNode {
    fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        match self {
            ASTNode::Float64(x) => x.eval(rt),
            ASTNode::Block(x) => x.eval(rt),
            ASTNode::SymbolicRef(x) => x.eval(rt),
            ASTNode::Neg(x) => x.eval(rt),
            ASTNode::Add(x) => x.eval(rt),
            ASTNode::Sub(x) => x.eval(rt),
            ASTNode::Mul(x) => x.eval(rt),
            ASTNode::Div(x) => x.eval(rt),
            ASTNode::Pow(x) => x.eval(rt),
            ASTNode::Function(x) => x.eval(rt),
            ASTNode::Call(x) => x.eval(rt),
            ASTNode::PlumRef(x) => x.eval(rt),
        }
    }
}

impl std::ops::Neg for ASTNode {
    type Output = ASTNode;
    fn neg(self) -> Self {
        ASTNode::Neg(Box::new(Neg { operand: self }))
    }
}

impl std::ops::Add for ASTNode {
    type Output = ASTNode;
    fn add(self, other: Self) -> Self {
        ASTNode::Add(Box::new(Add {
            lhs: self,
            rhs: other,
        }))
    }
}

impl std::ops::Sub for ASTNode {
    type Output = ASTNode;

    fn sub(self, other: Self) -> Self {
        ASTNode::Sub(Box::new(Sub {
            lhs: self,
            rhs: other,
        }))
    }
}

impl std::ops::Mul for ASTNode {
    type Output = ASTNode;

    fn mul(self, other: Self) -> Self {
        ASTNode::Mul(Box::new(Mul {
            lhs: self,
            rhs: other,
        }))
    }
}

impl std::ops::Div for ASTNode {
    type Output = ASTNode;

    fn div(self, other: Self) -> Self {
        ASTNode::Div(Box::new(Div {
            lhs: self,
            rhs: other,
        }))
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
        log::debug!("-- cos(1) using arithmetic expression macros --------------");
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
        log::debug!("-- cos(1) using std::ops arithmetic traits for better syntax --------------");
        let one = float64!(1.0);
        let cos_1 = one.clone()
            - float64!(1.0 / 2.0)
                * (one.clone()
                    - float64!(1.0 / (3.0 * 4.0))
                        * (one.clone()
                            - float64!(1.0 / (5.0 * 6.0))
                                * (one.clone()
                                    - float64!(1.0 / (7.0 * 8.0))
                                        * (one.clone() - float64!(1.0 / (9.0 * 10.0))))));
        log::debug!("cos_1 -> {:.17}", cos_1.eval(&mut rt)?.as_float64()?);
    }

    {
        log::debug!("-- block_0 --------------");
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
            define!(x_squared, symbolic_ref!(x) * symbolic_ref!(x));
            define!(z, float64!(1.0/(1.0*2.0)) * symbolic_ref!(x_squared));
            define!(y, float64!(1.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(3.0*4.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(5.0*6.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(7.0*8.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(9.0*10.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(11.0*12.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(13.0*14.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(15.0*16.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(17.0*18.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(19.0*20.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(21.0*22.0));
            sub_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(23.0*24.0));
            add_assign!(y, symbolic_ref!(z));
            mul_assign!(z, symbolic_ref!(x_squared) / float64!(25.0*26.0));
            ;
            symbolic_ref!(y)
        };
        let value = cos_2.eval(&mut rt)?.as_float64()?.as_f64();
        let error = (value - 2.0f64.cos()).abs();
        log::debug!(
            "cos_2 -> {:.17}; 'actual' value is {:.17}; error: {:.17e}",
            value,
            2.0f64.cos(),
            error,
        );
        assert!(error < 1.0e-16);
    }

    // Define some functions (which will be used in several places)
    let norm_squared_function = function!((x, y) -> block! {
        define!(z, symbolic_ref!(x) * symbolic_ref!(x));
        add_assign!(z, symbolic_ref!(y) * symbolic_ref!(y));
        ;
        symbolic_ref!(z)
    });
    let exp_function = function!((x) -> block! {
        define!(z, symbolic_ref!(x));
        define!(y, float64!(1.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(2.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(3.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(4.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(5.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(6.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(7.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(8.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(9.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(10.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(11.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(12.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(13.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(14.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(15.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(16.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(17.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(18.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(19.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(20.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(21.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(22.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(23.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(24.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(25.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(26.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x) / float64!(27.0));
        add_assign!(y, symbolic_ref!(z));
        ;
        symbolic_ref!(y)
    });
    let cos_function = function!((x) -> block! {
        define!(x_squared, symbolic_ref!(x) * symbolic_ref!(x));
        define!(z, symbolic_ref!(x_squared) / float64!(1.0*2.0));
        define!(y, float64!(1.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(3.0*4.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(5.0*6.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(7.0*8.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(9.0*10.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(11.0*12.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(13.0*14.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(15.0*16.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(17.0*18.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(19.0*20.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(21.0*22.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(23.0*24.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(25.0*26.0));
        ;
        symbolic_ref!(y)
    });
    let sin_function = function!((x) -> block! {
        define!(x_squared, mul!(symbolic_ref!(x), symbolic_ref!(x)));
        define!(z, symbolic_ref!(x));
        define!(y, symbolic_ref!(x));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(2.0*3.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(4.0*5.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(6.0*7.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(8.0*9.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(10.0*11.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(12.0*13.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(14.0*15.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(16.0*17.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(18.0*19.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(20.0*21.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(22.0*23.0));
        sub_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(24.0*25.0));
        add_assign!(y, symbolic_ref!(z));
        mul_assign!(z, symbolic_ref!(x_squared) / float64!(26.0*27.0));
        ;
        symbolic_ref!(y)
    });

    {
        log::debug!("-- exp function ----------------------------------------------------");
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
        let program_value = program.run_as_program(&mut rt)?.as_float64()?.as_f64();
        let actual_value = 1.0f64.exp();
        let error = (program_value - actual_value).abs();
        log::debug!(
            "program -> {:.17}; actual value: {:.17}; error: {:.17e}",
            program_value,
            actual_value,
            error,
        );
        assert!(error < 5.0e-16);
        log::debug!("rt.symbol_mv.len(): {}", rt.symbol_mv.len());
        log::debug!(
            "rt.symbol_mv.last().unwrap().keys():\n{:#?}",
            rt.symbol_mv.last().unwrap().keys()
        );
    }

    // Now create a Datahost and initialize the Datacache with it.
    let datahost_la = Arc::new(RwLock::new(idp_core::Datahost::open_in_memory(
        "PL".to_string(),
    )?));
    idp_core::initialize_datacache(idp_core::Datacache::new(datahost_la.clone()));

    // Store the functions in the Datahost
    let norm_squared_plum_head_seal = datahost_la.write().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&norm_squared_function)?)
            .build()?,
    )?;
    let exp_plum_head_seal = datahost_la.write().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&exp_function)?)
            .build()?,
    )?;
    let cos_plum_head_seal = datahost_la.write().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&cos_function)?)
            .build()?,
    )?;
    let sin_plum_head_seal = datahost_la.write().store_plum(
        &idp_proto::PlumBuilder::new()
            .with_body_content_type(idp_proto::ContentType::from("pl/function".as_bytes()))
            .with_body_content(rmp_serde::to_vec(&sin_function)?)
            .build()?,
    )?;

    // Here, create a program where the functions are linked in from hash-addressed Plums in the Datahost,
    // automatically loaded via PlumRef and Datacache.
    {
        log::debug!("------------------------------------------------------");
        rt.reset();
        let program = block! {
            define!(norm_squared, plum_ref!(norm_squared_plum_head_seal.clone().into()));
            define!(exp, plum_ref!(exp_plum_head_seal.clone().into()));
            define!(cos, plum_ref!(cos_plum_head_seal.clone().into()));
            define!(sin, plum_ref!(sin_plum_head_seal.clone().into()));
            ;
            call!(symbolic_ref!(norm_squared), (call!(symbolic_ref!(cos), (float64!(0.83))), call!(symbolic_ref!(sin), (float64!(0.83)))))
            // call!(symbolic_ref!(exp), (float64!(1.0)))
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
