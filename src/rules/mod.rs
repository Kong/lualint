use linked_hash_map::LinkedHashMap;

use std::sync::Mutex;

use downcast_rs::{impl_downcast, Downcast};
use full_moon::{
    ast::{lua52::Goto, *},
    tokenizer::{Position, Token, TokenReference},
};

macro_rules! rule_cast {
    ($rule: expr, $rule_type: path) => {
        match $rule {
            $rule_type(rule) => rule,
            _ => unreachable!(),
        }
    };
}

macro_rules! decl_rule {
    ($rule_name:ident, $description:expr, $version:expr, $config_example:expr) => {
        pub const RULE_NAME: &'static str = stringify!($rule_name);

        pub const RULE_INFO: RuleInfo = RuleInfo {
            name: RULE_NAME,
            description: $description,
            version: $version,
            config_example: $config_example,
        };

        pub fn init() {
            super::ALL_RULES.lock().unwrap().insert(RULE_NAME, RULE_INFO);
        }
    };
}
macro_rules! decl_rules {
    ($($mod_name:ident),+) => {
        $(pub mod $mod_name;)+
        pub fn init_all() {
            $( $mod_name::init(); )+
        }
    };
}

decl_rules!(
    eof_blank_line,
    func_separation,
    max_column_width,
    no_trailing_space,
    one_line_before_else,
    table_ctor_comma
);

pub struct RuleInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub config_example: &'static str,
}

lazy_static::lazy_static! {
    pub static  ref ALL_RULES: std::sync::Mutex<LinkedHashMap<&'static str, RuleInfo>> = {
        let map = LinkedHashMap::new();
        Mutex::new(map)
    };
}

#[derive(Clone, Debug)]
pub struct Pos {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(line: usize, column: usize) -> Pos {
        Pos { file: String::default(), line, col: column }
    }

    pub fn with_file(&mut self, file: String) -> Pos {
        self.file = file;
        self.to_owned()
    }
}
impl From<Position> for Pos {
    fn from(value: Position) -> Self {
        Self { file: String::default(), line: value.line(), col: value.character() }
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum NodeKey {
    Block,
    Eof,
    Stmt,
    Goto,
    LastStmt,
    Return,
    Break,
    ExprBlock,
    FuncCallBlock,
    TableConstructor,
    Assignment,
    Do,
    FuncCall,
    FuncDecl,
    GenericFor,
    NumericFor,
    If,
    LocalAssign,
    LocalFunc,
    Numericfor,
    Repeat,
    While,
    FuncArgsBlock,
    Field,
    TokenRef,
    FuncName,
}

pub enum NodeWrapper {
    Source(String),
    Token(Token),
    Goto(Goto),
    TokenRef(TokenReference),
    Block(Block),
    LastStmt(LastStmt),
    Return(Return),
    Expression(Expression),
    Value(Value),
    Stmt(Stmt),
    FunctionCall(FunctionCall),
    FunctionArgs(FunctionArgs),
    Assignment(Assignment),
    TableConstructor(TableConstructor),
    Do(Do),
    FunctionName(FunctionName),
    FunctionDeclaration(FunctionDeclaration),
    GenericFor(GenericFor),
    If(If),
    LocalAssignment(LocalAssignment),
    LocalFunction(LocalFunction),
    NumericFor(NumericFor),
    Repeat(Repeat),
    While(While),
}
pub enum WalkTy {
    Enter,
    Leave,
}
type RuleCallback = fn(rctx: &mut dyn RuleContext, rule: NodeWrapper) -> NodeWrapper;
type CallbackIndex = usize;
#[derive(Default)]
pub struct Registry {
    callbacks: Vec<RuleCallback>,
    enter_walker_map: LinkedHashMap<NodeKey, Vec<CallbackIndex>>,
    leave_walker_map: LinkedHashMap<NodeKey, Vec<CallbackIndex>>,
    token_listeners: Vec<CallbackIndex>,
    preprocessors: Vec<CallbackIndex>,
    pub rule_ctx: LinkedHashMap<String, Box<dyn RuleContext>>,
    callback_id_to_name: LinkedHashMap<CallbackIndex, String>,
}
pub trait RuleContext: Downcast {
    fn get_reports(&self) -> &Vec<LintReport>;
}

pub trait Rule {
    fn apply(rules: &mut Registry, config: &serde_json::Value) -> Self;
    fn context(&self) -> &dyn RuleContext;
}

impl_downcast!(RuleContext);

#[derive(Debug, Clone)]
pub enum ReportLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LintReport {
    pub pos: Pos,
    pub level: ReportLevel,
    pub msg: String,
}

impl Registry {
    pub fn get_ctx(&mut self, key: &str) -> Option<&mut dyn RuleContext> {
        if let Some(ctx) = self.rule_ctx.get_mut(key) {
            Some(ctx.as_mut())
        } else {
            None
        }
    }

    pub fn get_all_ctx(&self) -> &LinkedHashMap<String, Box<dyn RuleContext>> {
        &self.rule_ctx
    }
    pub fn bind_ctx(&mut self, rule_name: &str, ctx: Box<dyn RuleContext>) {
        self.rule_ctx.insert(rule_name.to_string(), ctx);
    }

    // pub fn register(&mut self, rule_name: &str, node_type: NodeKey, callback: Box<RuleCallback>) {
    //     let callback_index = self.callbacks.len();
    //     self.callbacks.push(callback);
    //     self.node_visitor_map
    //         .entry(node_type)
    //         .or_insert(vec![])
    //         .push(callback_index);
    //     self.callback_id_to_name.insert(callback_index, rule_name.to_string());
    // }

    pub fn register_walker(
        &mut self,
        rule_name: &str,
        node_type: NodeKey,
        walker_type: WalkTy,
        callback: RuleCallback,
    ) {
        let callback_index = self.callbacks.len();
        self.callbacks.push(callback);
        match walker_type {
            WalkTy::Enter => {
                self.enter_walker_map.entry(node_type).or_insert(vec![]).push(callback_index);
            }
            WalkTy::Leave => {
                self.leave_walker_map.entry(node_type).or_insert(vec![]).push(callback_index);
            }
        }
        self.callback_id_to_name.insert(callback_index, rule_name.to_string());
    }

    pub fn preprocess(&mut self, rule_name: &str, callback: RuleCallback) {
        let callback_index = self.callbacks.len();
        self.callbacks.push(callback);
        self.preprocessors.push(callback_index);
        self.callback_id_to_name.insert(callback_index, rule_name.to_string());
    }

    pub fn trigger_preprocess(&mut self, src: &str) -> String {
        let mut source = src.to_string();
        for callback in &self.preprocessors {
            let callback = *callback;
            let rule_name = self.callback_id_to_name.get(&callback).unwrap();
            let ctx: &mut dyn RuleContext = self.rule_ctx.get_mut(rule_name).unwrap().as_mut();
            let w = (self.callbacks[callback])(ctx, NodeWrapper::Source(source));
            source = match w {
                NodeWrapper::Source(s) => s,
                _ => unreachable!(),
            }
        }
        source
    }

    pub fn listen_token(&mut self, rule_name: &str, callback: RuleCallback) {
        let callback_index = self.callbacks.len();
        self.callbacks.push(callback);
        self.token_listeners.push(callback_index);
        self.callback_id_to_name.insert(callback_index, rule_name.to_string());
    }

    pub fn listen_enter(&mut self, rule_name: &str, node_type: NodeKey, callback: RuleCallback) {
        self.register_walker(rule_name, node_type, WalkTy::Enter, callback);
    }

    pub fn listen_leave(&mut self, rule_name: &str, node_type: NodeKey, callback: RuleCallback) {
        self.register_walker(rule_name, node_type, WalkTy::Leave, callback);
    }

    pub fn trigger_walker(
        &mut self,
        node_key: NodeKey,
        walker_type: WalkTy,
        rule: NodeWrapper,
    ) -> NodeWrapper {
        let mut rule = rule;
        let walker_map = match walker_type {
            WalkTy::Enter => &self.enter_walker_map,
            WalkTy::Leave => &self.leave_walker_map,
        };
        if let Some(callbacks) = walker_map.get(&node_key) {
            for callback in callbacks {
                let rule_name = self.callback_id_to_name.get(callback).unwrap();
                let ctx: &mut dyn RuleContext = self.rule_ctx.get_mut(rule_name).unwrap().as_mut();
                rule = (self.callbacks[*callback])(ctx, rule);
            }
        }
        rule
    }

    pub fn notify_enter(&mut self, node_key: NodeKey, rule: NodeWrapper) -> NodeWrapper {
        // println!("notify enter: {:?}", node_key);
        self.trigger_walker(node_key, WalkTy::Enter, rule)
    }

    pub fn notify_leave(&mut self, node_key: NodeKey, rule: NodeWrapper) -> NodeWrapper {
        // println!("notify leave: {:?}", node_key);
        self.trigger_walker(node_key, WalkTy::Leave, rule)
    }

    pub fn notify_token(&mut self, token: Token) -> Token {
        let mut token = token;
        for callback in &self.token_listeners {
            let rule_name = self.callback_id_to_name.get(callback).unwrap();
            let ctx: &mut dyn RuleContext = self.rule_ctx.get_mut(rule_name).unwrap().as_mut();
            let token_w = (self.callbacks[*callback])(ctx, NodeWrapper::Token(token));
            match token_w {
                NodeWrapper::Token(t) => {
                    token = t;
                }
                _ => {
                    panic!("token listener should return token");
                }
            }
        }
        token
    }
}
