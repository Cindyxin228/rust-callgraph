use rustc_hir::def::Res;
use rustc_hir::HirId;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::ParamEnvAnd;
use std::collections::{HashMap, HashSet};
use std::path;
use std::string;
use rustc_hir::intravisit;
use rustc_middle::hir::nested_filter;
use rustc_span::Span;
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use rustc_hir::LangItem;

macro_rules! skip_generated_code {
    ($span: expr) => {
        if $span.from_expansion() || $span.is_dummy() {
            return;
        }
    };
}

// Backup self.cur_fn, set cur_fn to id, continue to walk the AST by executing
// $walk, then restore self.cur_fn.
macro_rules! push_walk_pop {
    ($this: expr, $id: expr, $walk: expr) => {{
        let prev_fn = $this.cur_fn;
        $this.cur_fn = Some($id);
        $walk;
        $this.cur_fn = prev_fn;
    }};
}

#[derive( Debug, Clone)]
struct Call {
    // the call expression
    call_expr: HirId,
    call_expr_span: Span,
    // possible enclosing function
    caller: Option<DefId>,
    caller_span: Option<Span>,
    // call target
    callee: DefId,
    callee_span: Span,
    callee_path: String,
    constraint_depth: usize,
}

impl Eq for Call {}

impl Hash for Call {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.caller.hash(state);
        self.callee.hash(state);
    }
}

// 手动实现 PartialEq 只比较 caller 和 callee
impl PartialEq for Call {
    fn eq(&self, other: &Self) -> bool {
        self.caller == other.caller && self.callee_path == other.callee_path
    }
}

impl Call {
    fn should_insert(&self, new_depth: usize) -> bool {
        new_depth < self.constraint_depth
    }
}

pub struct CallgraphVisitor<'tcx> {
    // type context
    tcx: TyCtxt<'tcx>,

    // free functions
    functions: HashSet<(DefId, Span)>,
    // trait method declarations without default implementation
    method_decls: HashSet<DefId>,
    // map decls to impls
    method_impls: HashMap<DefId, Vec<DefId>>,

    // static calls
    static_calls: HashSet<Call>,
    // dynamic calls
    dynamic_calls: HashSet<Call>,
    //non local calls
    non_local_calls: HashSet<Call>,

    // tracks the current function we're in during AST walk
    cur_fn: Option<DefId>,

    // 新增字段来跟踪约束层数
    constraint_depth: usize,

    enter_if: bool,

}

impl<'tcx> CallgraphVisitor<'tcx> {
    pub fn new(tcx: &TyCtxt<'tcx>) -> CallgraphVisitor<'tcx> {
        CallgraphVisitor {
            tcx: *tcx,
            functions: HashSet::new(),
            method_decls: HashSet::new(),
            method_impls: HashMap::new(),
            static_calls: HashSet::new(),
            dynamic_calls: HashSet::new(),
            non_local_calls: HashSet::new(),
            cur_fn: None,
            constraint_depth: 0,
            enter_if: false,
        }
    }

    pub fn dump(&self) {
        println!("Functions:");
        for (def_id, span) in &self.functions {
            let function_name = self.tcx.def_path_str(*def_id);
            println!("  Function: {}, Span: {:?}", function_name, span);
        }

        println!("\nMethod Declarations:");
        for def_id in &self.method_decls {
            let method_name = self.tcx.def_path_str(*def_id);
            println!("  Method Declaration: {}", method_name);
        }

        println!("\nMethod Implementations:");
        for (decl_id, impl_ids) in &self.method_impls {
            let decl_name = self.tcx.def_path_str(*decl_id);
            println!("  Method Implementation for {}: {:?}", decl_name, impl_ids);
        }

        println!("\nStatic Calls:");
        for call in &self.static_calls {
            let caller_str = match call.caller {
                Some(caller) => self.tcx.def_path_str(caller),
                None => "Unknown Caller".to_string(),
            };
            let callee_str = self.tcx.def_path_str(call.callee);

            println!("{} --- {} (Constraint Depth: {})", caller_str, call.callee_path, call.constraint_depth);
        }

        println!("\nDynamic Calls:");
        for call in &self.dynamic_calls {
            let caller_str = match call.caller {
                Some(caller) => self.tcx.def_path_str(caller),
                None => "Unknown Caller".to_string(),
            };
            let callee_str = self.tcx.def_path_str(call.callee);

            println!("{} --- {} (Constraint Depth: {})", caller_str, callee_str, call.constraint_depth);
        }

        println!("\nNon Local Calls:");
        for call in &self.non_local_calls {
            let caller_str = match call.caller {
                Some(caller) => self.tcx.def_path_str(caller),
                None => "Unknown Caller".to_string(),
            };

            println!("{} --- {} (Constraint Depth: {})", caller_str, call.callee_path, call.constraint_depth);
        }
    }

    fn handle_call(&mut self, new_call: Call, call_type: String) {
        if call_type == "static"{
            if let Some(existing_call) = self.static_calls.get(&new_call).cloned() {
                if existing_call.should_insert(self.constraint_depth) {
                    self.static_calls.remove(&existing_call);
                    self.static_calls.insert(new_call);
                    //println!("Updated call with new constraint depth: {}", self.constraint_depth);
                }
            } else {
                self.static_calls.insert(new_call);
                //println!("Inserted new call with constraint depth: {}", self.constraint_depth);
            }
        }
        else if call_type == "dynamic"{
            if let Some(existing_call) = self.dynamic_calls.get(&new_call).cloned() {
                if existing_call.should_insert(self.constraint_depth) {
                    self.dynamic_calls.remove(&existing_call);
                    self.dynamic_calls.insert(new_call);
                    //println!("Updated call with new constraint depth: {}", self.constraint_depth);
                }
            } else {
                self.dynamic_calls.insert(new_call);
                //println!("Inserted new call with constraint depth: {}", self.constraint_depth);
            }
        }
        else{
            if let Some(existing_call) = self.non_local_calls.get(&new_call).cloned() {
                if existing_call.should_insert(self.constraint_depth) {
                    self.non_local_calls.remove(&existing_call);
                    self.non_local_calls.insert(new_call);
                    //println!("Updated call with new constraint depth: {}", self.constraint_depth);
                }
            } else {
                self.non_local_calls.insert(new_call);
                //println!("Inserted new call with constraint depth: {}", self.constraint_depth);
            }
        }
        
    }

    fn process_method_call(&mut self, hir_id: HirId, segment: &&rustc_hir::PathSegment<'_>, expr: &'tcx rustc_hir::Expr){
        let o_def_id = hir_id.owner;
                let typeck_tables = self.tcx.typeck(o_def_id);
                let substs = typeck_tables.node_args(hir_id);
            
                // 获取方法调用的定义 ID
                let method_id = typeck_tables.type_dependent_def_id(hir_id);
            
                match method_id {
                    Some(def_id) => {
                        // 静态分发：已知具体的实现
                        if let Some(callid) = self.cur_fn{
                            let param_env = self.tcx.param_env(callid);
            
                        match self.tcx.resolve_instance_raw(ParamEnvAnd { param_env, value: (def_id, substs) }) {
                            Ok(Some(inst)) => {
                                // 成功解析为具体的实例
                                let res_def_id = inst.def_id();
                            // println!("caller: {:?}", self.cur_fn);
                            // println!("def_id: {:?}, get_path: {:#?}", res_def_id, self.tcx.def_path_str(res_def_id));
                            match self.tcx.hir().get_if_local(res_def_id) {
                                Some(rustc_hir::Node::TraitItem(rustc_hir::TraitItem { span, .. })) => {
                                    // dynamic calls resolve only to the trait method decl
                                    let new_call = Call {
                                        call_expr: hir_id,
                                        call_expr_span: expr.span,
                                        caller: self.cur_fn,
                                        caller_span: None,
                                        callee: res_def_id,
                                        callee_span: *span,
                                        callee_path: self.tcx.def_path_str(res_def_id),
                                        constraint_depth: self.constraint_depth,
                                    };
                                    self.handle_call(new_call, "dynamic".to_string());
                                }
                                Some(rustc_hir::Node::ImplItem(rustc_hir::ImplItem { span, .. })) |
                                Some(rustc_hir::Node::Item(rustc_hir::Item { span, .. })) |
                                Some(rustc_hir::Node::ForeignItem(rustc_hir::ForeignItem { span, .. })) => {
                                    // calls for which the receiver's type can be resolved
                                    let new_call = Call {
                                        call_expr: hir_id,
                                        call_expr_span: expr.span,
                                        caller: self.cur_fn,
                                        caller_span: None,
                                        callee: res_def_id,
                                        callee_span: *span,
                                        callee_path: self.tcx.def_path_str(res_def_id),
                                        constraint_depth: self.constraint_depth,
                                    };

                                    self.handle_call(new_call, "static".to_string());
                                }
                                None => {
                                    let new_call = Call {
                                        call_expr: hir_id,
                                        call_expr_span: expr.span,
                                        caller: self.cur_fn,
                                        caller_span: None,
                                        callee: res_def_id,
                                        callee_span: Span::default(),
                                        callee_path: self.tcx.def_path_str(res_def_id),
                                        constraint_depth: self.constraint_depth,
                                    };

                                    self.handle_call(new_call, "non_local".to_string());
                                },
                                _ => todo!()
                                };
                            },
                            Ok(None) | Err(_) => {
                                // 无法解析为具体实例，可能是动态分发的调用
                                let new_call = Call {
                                    call_expr: hir_id,
                                    call_expr_span: expr.span,
                                    caller: self.cur_fn,
                                    caller_span: None,
                                    callee: def_id,
                                    callee_span: expr.span,
                                    callee_path: self.tcx.def_path_str(def_id),
                                    constraint_depth: self.constraint_depth,
                                };
            
                                println!("new dynamic call: {:#?}", new_call);
                                self.handle_call(new_call, "dynamic".to_string());
                            }
                        }
                    }
                    }
                    None => {
                        // 动态分发：无法直接解析具体的实现
                        let new_call = Call {
                            call_expr: hir_id,
                            call_expr_span: expr.span,
                            caller: self.cur_fn,
                            caller_span: None,
                            callee: segment.res.def_id(),
                            callee_span: expr.span,
                            callee_path: self.tcx.def_path_str(segment.res.def_id()),
                            constraint_depth: self.constraint_depth,
                        };
            
                        println!("new dynamic call: {:#?}", new_call);
                        self.handle_call(new_call, "dynamic".to_string());
                    }
                }
            
                intravisit::walk_expr(self, expr); // 确保遍历所有表达式
    }

    fn process_call(&mut self, hir_id: HirId, qpath: &rustc_hir::QPath, expr: &'tcx rustc_hir::Expr){
        match qpath {
            rustc_hir::QPath::Resolved(_, p) => {
                // println!("Resolved path: {:?}", p); // 打印解析后的路径信息
                if let rustc_hir::def::Res::Def(_, def_id) = p.res {
                    let new_call = Call {
                        call_expr: hir_id,
                        call_expr_span: expr.span,
                        caller: self.cur_fn,
                        caller_span: None,
                        callee: def_id,
                        callee_span: p.span,
                        callee_path: self.tcx.def_path_str(def_id),
                        constraint_depth: self.constraint_depth,
                    };

                    println!("resolved new call {:?}", new_call);
        
                    // 检查是否已经存在相同的调用（只比较 caller 和 callee）
                    self.handle_call(new_call, "static".to_string());
                }
            }
            rustc_hir::QPath::TypeRelative(ty, path_segment) => {   
                // println!("TypeRelative path: {:?}", ty);
                if let rustc_hir::TyKind::Path(ref qpath) = ty.kind {
                    if let rustc_hir::QPath::Resolved(_, path) = qpath {
                        if let rustc_hir::def::Res::Def(_, def_id) = path.res {
                            // Convert DefId and Ident to strings for printing
                            let def_id_str = self.tcx.def_path_str(def_id);
                            let ident_str = path_segment.ident.to_string();
                            let callee_path_output = def_id_str + "::" + &ident_str;
                            let new_call = Call {
                                call_expr: hir_id,
                                call_expr_span: expr.span,
                                caller: self.cur_fn,
                                caller_span: None,
                                callee: def_id,
                                callee_span: path_segment.ident.span,   //error span
                                callee_path: callee_path_output.clone(),
                                constraint_depth: self.constraint_depth,
                            };
                            // println!("Typeratived new call {:?}", new_call);
                    
                            self.handle_call(new_call, "static".to_string());
                        }
                    }
                }
            }
                
                
            rustc_hir::QPath::LangItem(_, span) => {
                println!("LangItem path: {:?}", span); // 打印语言项路径信息
            }
        }
        
        intravisit::walk_expr(self, expr); // 确保遍历所有表达式
    }

}



//解析函数调用，存储静态调用和动态调用
impl<'tcx> intravisit::Visitor<'tcx> for CallgraphVisitor<'tcx> {

    //only see the body, ignore other information
    type NestedFilter = nested_filter::OnlyBodies;
    

    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

   

    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr) {
        // skip_generated_code!(expr.span);

        let old_depth = self.constraint_depth; // 保存当前深度
        let hir_id = expr.hir_id;
        let mut flag = true;
        println!("The code is {:#?}", self.tcx.sess.source_map().span_to_snippet(expr.span));
        // println!("Entering expr: {:#?}, constraint_depth{}", expr.kind, self.constraint_depth);
        // 检查表达式类型并更新约束层数
        match expr.kind {
            rustc_hir::ExprKind::If(ref cond, ref then , ref else_ex) => {
                self.enter_if = true;
                intravisit::walk_expr(self, cond); // 处理条件表达式
                self.enter_if = false;
                match self.tcx.sess.source_map().span_to_snippet(expr.span) {
                    Ok(snippet) => {
                        if snippet.contains("debug") {  //应对debug_assert
                        }
                        else{
                            self.constraint_depth += 1;
                        }
                    }
                    _=>{
                        self.constraint_depth += 1;
                    }
                }
                intravisit::walk_expr(self, then); // 处理条件表达式
                // println!("into if expr:{:#?}, constraint:{}", expr, self.constraint_depth);
                else_ex.as_ref().map(|expr| intravisit::walk_expr(self, expr));
                
            },
            rustc_hir::ExprKind::Binary(op, ref lhs, ref rhs) => {
                // 处理逻辑运算符
                // println!("into binary, depth {:?}", self.constraint_depth);
                flag = false;
                if self.enter_if{ 
                    match op.node {
                        rustc_hir::BinOpKind::And => {
                            //println!("Entering And lhs, constraint_depth is {}, code: {:?}", self.constraint_depth, self.tcx.sess.source_map().span_to_snippet(lhs.span));
                            self.visit_expr(lhs);
                            self.constraint_depth += 1; // 每个 and 增加一个约束深度
                            //println!("Entering And rhs, constraint_depth is {}, code: {:?}", self.constraint_depth, self.tcx.sess.source_map().span_to_snippet(rhs.span));
                            self.visit_expr(rhs);
                            // println!("And expr:{:#?}, constraint:{}", expr, self.constraint_depth);
                        },
                        rustc_hir::BinOpKind::Or => {
                            // 对于 or，计算左右子表达式的约束深度
                            let left_depth = self.constraint_depth;
                            // intravisit::walk_expr(self, lhs); // 递归访问左侧表达式
                            self.visit_expr(lhs);
                            let left_constraint_depth = self.constraint_depth;

                            self.constraint_depth = left_depth; // 恢复之前的深度
                            self.visit_expr(rhs);
                            let right_constraint_depth = self.constraint_depth;
                            
                            // 选择最小的约束深度
                            self.constraint_depth = left_constraint_depth.min(right_constraint_depth);
                            // println!("Or expr:{:#?}, constraint:{}", expr, self.constraint_depth);
                        },
                        _ => {
                            // 处理其他二元运算符
                            self.visit_expr(lhs);
                            self.visit_expr(rhs);
                        }
                    }
                }
                else{
                    self.visit_expr(lhs);
                    self.visit_expr(rhs);
                }
            },
            rustc_hir::ExprKind::Match(ref match_expr, _, match_source) => {
                let not_for_loop_match = match match_expr.kind {
                    rustc_hir::ExprKind::Call(ref callee, _) => {
                        println!("into Match call");
                        // 检查 callee 是否是 `next()` 方法，这通常是 `for` 循环的一部分
                        if let rustc_hir::ExprKind::Path(rustc_hir::QPath::LangItem(
                            LangItem::IteratorNext,
                            _,
                        )) = callee.kind
                        {
                            // 如果是 `next()` 方法调用，则它可能是 `for` 循环的一部分
                            println!("is inner Match");
                            false
                        } else {
                            true
                        }
                    }
                    _ => true,
                };

                let is_real_match = match match_source {
                    | rustc_hir::MatchSource::TryDesugar(_) // 注意 TryDesugar 包含一个 HirId
                    | rustc_hir::MatchSource::AwaitDesugar
                    | rustc_hir::MatchSource::FormatArgs => false,
                    _ => true,
                };

                let is_real_match = is_real_match & not_for_loop_match;
                
                
            
                if is_real_match {
                    // 如果不是 `for` 循环的内层 match或desugar的match，则增加深度
                    self.constraint_depth += 1; // 进入 match 语句
                }
                
                // println!("Match expr:{:#?}, constraint:{}", expr, self.constraint_depth);
                intravisit::walk_expr(self, expr); // 确保遍历所有表达式
                // println!("Match expr:{:#?}, constraint:{}", expr, self.constraint_depth);
            },
            rustc_hir::ExprKind::Call(
                rustc_hir::Expr {
                    kind: rustc_hir::ExprKind::Path(ref qpath),
                    ..
                },
                _,
            ) => {
                // println!("call path {:?}", qpath);
                self.process_call(hir_id, qpath, expr);
            },
            rustc_hir::ExprKind::MethodCall(ref segment, ref receiver, ref args, span) => {
                self.process_method_call(hir_id, segment, expr);
            },
            _ => {
                //println!("Processing other expression: {:?}", expr);
                intravisit::walk_expr(self, expr); // 确保遍历所有表达式
            }  
        }
        if flag {
            self.constraint_depth = old_depth; // 恢复之前的深度
            //println!("Exiting expression {:?}, restored constraint_depth to {}", expr, self.constraint_depth);
        }
    }

    //解析函数定义，存储函数信息
    fn visit_item(&mut self, item: &'tcx rustc_hir::Item) {
        skip_generated_code!(item.span);

        let hir_id = item.hir_id();
        if let rustc_hir::ItemKind::Fn(_, _, _) = item.kind {
            let def_id = hir_id.owner.to_def_id();
            self.functions.insert((def_id, item.span));

            push_walk_pop!(self, def_id, intravisit::walk_item(self, item));

            return;
        }
        // traverse
        intravisit::walk_item(self, item)
    }


    //解析trait定义，存储trait方法声明
    fn visit_trait_item(&mut self, ti: &'tcx rustc_hir::TraitItem) {
        skip_generated_code!(ti.span); // TODO ?do we want this

        let hir_id = ti.hir_id();
        let def_id = hir_id.owner.to_def_id();

        match ti.kind {
            rustc_hir::TraitItemKind::Fn(_, rustc_hir::TraitFn::Required(_)) => {
                // a method declaration
                self.method_decls.insert(def_id);
                self.method_impls.insert(def_id, vec![]);
            }
            rustc_hir::TraitItemKind::Fn(_, rustc_hir::TraitFn::Provided(_)) => {
                // a method decl and def
                self.method_decls.insert(def_id);
                self.functions.insert((def_id, ti.span));
                self.method_impls.entry(def_id).or_default().push(def_id);

                push_walk_pop!(self, def_id, intravisit::walk_trait_item(self, ti));

                return;
            }
            _ => {}
        }

        // traverse
        intravisit::walk_trait_item(self, ti)
    }

    // self.tcx.hir().hir_to_pretty_string(ty.hir_id)

    //解析impl定义，存储impl方法实现，并链接trait方法声明
    fn visit_impl_item(&mut self, ii: &'tcx rustc_hir::ImplItem) {
        skip_generated_code!(ii.span);

        let hir_id = ii.hir_id();
        let def_id = hir_id.owner.to_def_id();

        if let rustc_hir::ImplItemKind::Fn(..) = ii.kind {
            self.functions.insert((def_id, ii.span));

            // store link to decl
            let mut decl_id = None;
            if let Some(impl_id) = self.tcx.impl_of_method(def_id) {
                if let Some(rustc_hir::Node::Item(item)) = self.tcx.hir().get_if_local(impl_id) {
                    if let rustc_hir::ItemKind::Impl(..) = item.kind {
                        // the next one filters methods that are just associated
                        // and do not belong to a struct
                        if let Some(trait_def_id) = self.tcx.trait_id_of_impl(impl_id) {
                            let item = self.tcx
                                .associated_items(trait_def_id)
                                .filter_by_name_unhygienic(ii.ident.name)
                                .next(); // There should ideally be only one item matching the name
                            if let Some(item) = item {
                                decl_id = Some(item.def_id);
                            };
                        }
                    }
                }
            }

            if let Some(decl_def_id) = decl_id {
                self.method_impls
                    .entry(decl_def_id)
                    .or_default()
                    .push(def_id);
            }

            push_walk_pop!(self, def_id, intravisit::walk_impl_item(self, ii));

            return;
        }

        // traverse
        intravisit::walk_impl_item(self, ii)
    }
}



