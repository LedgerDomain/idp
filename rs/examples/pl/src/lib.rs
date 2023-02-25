use anyhow::Result;
use async_lock::RwLock;
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

#[async_trait::async_trait]
pub trait Executable {
    async fn exec(&self, rt: &mut Runtime) -> Result<()>;
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

#[async_trait::async_trait]
impl Executable for Statement {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        match self {
            Statement::Definition(x) => x.exec(rt).await,
            Statement::Assignment(x) => x.exec(rt).await,
            Statement::AddAssignment(x) => x.exec(rt).await,
            Statement::SubAssignment(x) => x.exec(rt).await,
            Statement::MulAssignment(x) => x.exec(rt).await,
            Statement::DivAssignment(x) => x.exec(rt).await,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Definition {
    pub symbol_id: String,
    pub expr: ASTNode,
}

#[async_trait::async_trait]
impl Executable for Definition {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        rt.define(self.symbol_id.clone(), value)
    }
}

#[macro_export]
macro_rules! define {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::Definition(Box::new($crate::Definition {
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

#[async_trait::async_trait]
impl Executable for Assignment {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write().await = value;
        Ok(())
    }
}

#[macro_export]
macro_rules! assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::Assignment(Box::new($crate::Assignment {
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

#[async_trait::async_trait]
impl Executable for AddAssignment {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write().await += value;
        Ok(())
    }
}

#[macro_export]
macro_rules! add_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::AddAssignment(Box::new($crate::AddAssignment {
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

#[async_trait::async_trait]
impl Executable for SubAssignment {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write().await -= value;
        Ok(())
    }
}

#[macro_export]
macro_rules! sub_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::SubAssignment(Box::new($crate::SubAssignment {
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

#[async_trait::async_trait]
impl Executable for MulAssignment {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write().await *= value;
        Ok(())
    }
}

#[macro_export]
macro_rules! mul_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::MulAssignment(Box::new($crate::MulAssignment {
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

#[async_trait::async_trait]
impl Executable for DivAssignment {
    async fn exec(&self, rt: &mut Runtime) -> Result<()> {
        let value = self.expr.eval(rt).await?;
        let symbol_value_la = rt.dereference(self.symbol_id.as_str())?;
        *symbol_value_la.write().await /= value;
        Ok(())
    }
}

#[macro_export]
macro_rules! div_assign {
    ($symbol_id: ident, $expr: expr $(,)?) => {
        $crate::Statement::DivAssignment(Box::new($crate::DivAssignment {
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
    pub async fn run_as_program(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        assert_eq!(rt.symbol_mv.len(), 1, "Block::run_as_program can only be used when the Runtime's symbol table stack is depth 1");
        self.eval_impl(rt, false).await
    }
    async fn eval_impl(
        &self,
        rt: &mut Runtime,
        use_inner_stack_guard: bool,
    ) -> Result<ConcreteValue> {
        if use_inner_stack_guard {
            let stack_guard = StackGuard::new(rt);
            for statement in self.statement_v.iter() {
                statement.exec(stack_guard.rt).await?;
            }
            let value = self.expr.eval(stack_guard.rt).await?;
            Ok(value)
        } else {
            for statement in self.statement_v.iter() {
                statement.exec(rt).await?;
            }
            self.expr.eval(rt).await
        }
    }
}

#[async_trait::async_trait]
impl Expr for Block {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        self.eval_impl(rt, true).await
    }
}

/// NOTE/TODO: I couldn't figure out how to get it to parse Rust-style block syntax
/// like `block! { x; y; z; value }`, instead, for now, you have to do
/// `block! { x; y; z;; value }`.
/// TODO: Maybe need to put the more complex rule first, since apparently macro parsing
/// is very simple and can't do any backtracking.
#[macro_export]
macro_rules! block {
    ($x:expr) => {
        $crate::ASTNode::Block(Box::new($crate::Block {
            statement_v: Vec::new(),
            expr: $x,
        }))
    };
    ($($statement:expr;)+; $x:expr) => {
        $crate::ASTNode::Block(Box::new($crate::Block {
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

#[async_trait::async_trait]
pub trait Expr {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue>;
}

#[async_trait::async_trait]
impl Expr for idp_core::PlumRef<ASTNode> {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // The call to value() will handle loading, deserializing, and caching into memory.
        self.value_a().await?.eval(rt).await
    }
}

#[macro_export]
macro_rules! plum_ref {
    ($plum_head_seal:expr) => {
        $crate::ASTNode::PlumRef(Box::new(idp_core::PlumRef::<$crate::ASTNode>::new(
            $plum_head_seal,
        )))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Function {
    /// All arguments are implicitly typed as f64 for now.
    pub argument_name_v: Vec<String>,
    // TODO: return type (for now, this is implicitly f64)
    pub body: ASTNode,
}

#[async_trait::async_trait]
impl Expr for Function {
    async fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        // Inefficient, but fine for now.
        Ok(ConcreteValue::Function(self.clone()))
    }
}

#[macro_export]
macro_rules! function {
    (($($argument_name_v:ident),*) -> $body:expr) => {
        $crate::ASTNode::Function(Box::new($crate::Function {
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

#[async_trait::async_trait]
impl Expr for Call {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let function_eval = self.function.eval(rt).await?;
        let function = function_eval.as_function()?;
        let stack_guard = StackGuard::new(rt);
        for (argument_name, argument_expr) in
            std::iter::zip(function.argument_name_v.iter(), self.argument_expr_v.iter())
        {
            let argument_value = argument_expr.eval(stack_guard.rt).await?;

            stack_guard
                .rt
                .define(argument_name.clone(), argument_value)?;
        }
        let retval = function.body.eval(stack_guard.rt).await?;
        Ok(retval)
    }
}

#[macro_export]
macro_rules! call {
    ($function:expr, ($($argument_expr_v:expr),*)) => {
        $crate::ASTNode::Call(Box::new($crate::Call {
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
pub struct Float64(pub f64);

impl Float64 {
    pub fn as_f64(self) -> f64 {
        self.0
    }
}

#[async_trait::async_trait]
impl Expr for Float64 {
    async fn eval(&self, _rt: &mut Runtime) -> Result<ConcreteValue> {
        Ok(ConcreteValue::Float64(*self))
    }
}

#[macro_export]
macro_rules! float64 {
    ($x: expr) => {
        $crate::ASTNode::Float64($crate::Float64($x))
    };
}

#[derive(
    Clone,
    Debug,
    derive_more::Deref,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
    serde::Serialize,
)]
pub struct SymbolicRef(String);

#[async_trait::async_trait]
impl Expr for SymbolicRef {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        // This is inefficient, but is fine for now.  The solution would be to return
        // Arc<RwLock<ConcreteValue>> from Expr::eval, but that has its own drawbacks.
        Ok(rt.dereference(self.as_str())?.read().await.clone())
    }
}

#[macro_export]
macro_rules! symbolic_ref {
    ($symbol_id: ident) => {
        $crate::ASTNode::SymbolicRef($crate::SymbolicRef::from(
            stringify!($symbol_id).to_string(),
        ))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neg {
    pub operand: ASTNode,
}

#[async_trait::async_trait]
impl Expr for Neg {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let x = self.operand.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(-x)))
    }
}

#[macro_export]
macro_rules! neg {
    ($x: expr) => {
        $crate::ASTNode::Neg(Box::new($crate::Neg($x)))
    };
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Add {
    pub lhs: ASTNode,
    pub rhs: ASTNode,
}

#[async_trait::async_trait]
impl Expr for Add {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt).await?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs + rhs)))
    }
}

#[macro_export]
macro_rules! add {
    ($lhs: expr, $rhs: expr $(,)?) => {
        $crate::ASTNode::Add(Box::new($crate::Add {
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

#[async_trait::async_trait]
impl Expr for Sub {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt).await?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs - rhs)))
    }
}

#[macro_export]
macro_rules! sub {
    ($lhs: expr, $rhs: expr $(,)?) => {
        $crate::ASTNode::Sub(Box::new($crate::Sub {
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

#[async_trait::async_trait]
impl Expr for Mul {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt).await?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs * rhs)))
    }
}

#[macro_export]
macro_rules! mul {
    ($lhs: expr, $rhs: expr $(,)?) => {
        $crate::ASTNode::Mul(Box::new($crate::Mul {
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

#[async_trait::async_trait]
impl Expr for Div {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let lhs = self.lhs.eval(rt).await?.as_float64()?.as_f64();
        let rhs = self.rhs.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(lhs / rhs)))
    }
}

#[macro_export]
macro_rules! div {
    ($lhs: expr, $rhs: expr $(,)?) => {
        $crate::ASTNode::Div(Box::new($crate::Div {
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

#[async_trait::async_trait]
impl Expr for Pow {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        let base = self.base.eval(rt).await?.as_float64()?.as_f64();
        let exponent = self.exponent.eval(rt).await?.as_float64()?.as_f64();
        Ok(ConcreteValue::Float64(Float64(base.powf(exponent))))
    }
}

#[macro_export]
macro_rules! pow {
    ($base: expr, $exponent: expr $(,)?) => {
        $crate::ASTNode::Pow(Box::new($crate::Pow {
            base: $base,
            exponent: $exponent,
        }))
    };
}

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

#[async_trait::async_trait]
impl Expr for ASTNode {
    async fn eval(&self, rt: &mut Runtime) -> Result<ConcreteValue> {
        match self {
            ASTNode::Float64(x) => x.eval(rt).await,
            ASTNode::Block(x) => x.eval(rt).await,
            ASTNode::SymbolicRef(x) => x.eval(rt).await,
            ASTNode::Neg(x) => x.eval(rt).await,
            ASTNode::Add(x) => x.eval(rt).await,
            ASTNode::Sub(x) => x.eval(rt).await,
            ASTNode::Mul(x) => x.eval(rt).await,
            ASTNode::Div(x) => x.eval(rt).await,
            ASTNode::Pow(x) => x.eval(rt).await,
            ASTNode::Function(x) => x.eval(rt).await,
            ASTNode::Call(x) => x.eval(rt).await,
            ASTNode::PlumRef(x) => x.eval(rt).await,
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
