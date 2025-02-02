use rustc_driver::pretty::print;
use rustc_hir::HirId;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::ParamEnvAnd;
use std::collections::{HashMap, HashSet};
use rustc_hir::intravisit;
use rustc_middle::hir::nested_filter;
use rustc_span::Span;
use std::cmp::PartialEq;

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

#[derive(Hash, Eq, Debug, Clone)]
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
    constraint_depth: usize,
}

// 手动实现 PartialEq 只比较 caller 和 callee
impl PartialEq for Call {
    fn eq(&self, other: &Self) -> bool {
        self.caller == other.caller && self.callee == other.callee
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

    // tracks the current function we're in during AST walk
    cur_fn: Option<DefId>,

    // 新增字段来跟踪约束层数
    constraint_depth: usize,
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
            cur_fn: None,
            constraint_depth: 0,
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

            println!("{} --- {} (Constraint Depth: {})", caller_str, callee_str, call.constraint_depth);
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
        skip_generated_code!(expr.span);

        let old_depth = self.constraint_depth; // 保存当前深度
        let hir_id = expr.hir_id;
        let mut flag = true;
        // 检查表达式类型并更新约束层数
        match expr.kind {
            rustc_hir::ExprKind::If(ref cond, _, _) => {
                self.constraint_depth += 1; // 进入 if 语句
                //println!("Entering If {:?}, constraint_depth is {}", expr, self.constraint_depth);

                intravisit::walk_expr(self, cond); // 处理条件表达式
                // self.visit_expr(cond);
                // 处理 if 语句体
                // 在这里可以插入调用的逻辑
                // 例如，您可以在这里检查并存储调用
            },
            rustc_hir::ExprKind::Binary(op, ref lhs, ref rhs) => {
                // 处理逻辑运算符
                flag = false;
                match op.node {
                    rustc_hir::BinOpKind::And => {
                        self.constraint_depth += 1; // 每个 and 增加一个约束深度
                        println!("Entering And , constraint_depth is {}", self.constraint_depth);
                        intravisit::walk_expr(self, lhs); // 递归访问左侧表达式
                        intravisit::walk_expr(self, rhs); // 递归访问右侧表达式
                    },
                    rustc_hir::BinOpKind::Or => {
                        // 对于 or，计算左右子表达式的约束深度
                        let left_depth = self.constraint_depth;
                        intravisit::walk_expr(self, lhs); // 递归访问左侧表达式
                        let left_constraint_depth = self.constraint_depth;

                        self.constraint_depth = left_depth; // 恢复之前的深度
                        intravisit::walk_expr(self, rhs); // 递归访问右侧表达式
                        let right_constraint_depth = self.constraint_depth;

                        // 选择最小的约束深度
                        self.constraint_depth = left_constraint_depth.min(right_constraint_depth);
                        println!("After Or {:?}, constraint_depth is {}", expr, self.constraint_depth);
                    },
                    _ => {
                        // 处理其他二元运算符
                        intravisit::walk_expr(self, lhs);
                        intravisit::walk_expr(self, rhs);
                    }
                }
            },
            rustc_hir::ExprKind::Loop(..) => {
                if self.constraint_depth == 0 {
                    self.constraint_depth += 1; // 进入 loop 语句
                }
            },
            rustc_hir::ExprKind::Match(..) => {
                self.constraint_depth += 1; // 进入 match 语句
            },
            rustc_hir::ExprKind::Call(
                rustc_hir::Expr {
                    kind: rustc_hir::ExprKind::Path(ref qpath),
                    ..
                },
                _,
            ) => {
                // println!("Processing call with qpath: {:?}", qpath); // 打印 qpath 信息

                // if let rustc_hir::QPath::Resolved(_, p) = qpath {
                //     println!("Resolved path: {:?}", p); // 打印解析后的路径信息
                //     if let rustc_hir::def::Res::Def(_, def_id) = p.res {
                //         let new_call = Call {
                //             call_expr: hir_id,
                //             call_expr_span: expr.span,
                //             caller: self.cur_fn,
                //             caller_span: None,
                //             callee: def_id,
                //             callee_span: p.span,
                //             constraint_depth: self.constraint_depth,
                //         };

                //         // 检查是否已经存在相同的调用（只比较 caller 和 callee）
                //         if let Some(existing_call) = self.static_calls.get(&new_call).cloned() {
                //             // 如果存在相同的 caller 和 callee，比较约束深度
                //             if existing_call.should_insert(self.constraint_depth) {
                //                 // 移除现有的调用
                //                 self.static_calls.remove(&existing_call);
                //                 // 插入新的调用
                //                 self.static_calls.insert(new_call);
                //                 println!("Replaced existing call with new call, constraint_depth: {}", self.constraint_depth);
                //             }
                //         } else {
                //             self.static_calls.insert(new_call);
                //             println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                //         }
                //     }
                // } else {
                //     println!("Call expression is not resolved: {:?}", qpath); // 打印未解析的路径信息
                // }
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
                                constraint_depth: self.constraint_depth,
                            };
                
                            // 检查是否已经存在相同的调用（只比较 caller 和 callee）
                            if let Some(existing_call) = self.static_calls.get(&new_call).cloned() {
                                // 如果存在相同的 caller 和 callee，比较约束深度
                                if existing_call.should_insert(self.constraint_depth) {
                                    // 移除现有的调用
                                    self.static_calls.remove(&existing_call);
                                    // 插入新的调用
                                    self.static_calls.insert(new_call);
                                    println!("Replaced existing call with new call, constraint_depth: {}", self.constraint_depth);
                                }
                            } else {
                                self.static_calls.insert(new_call);
                                println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                            }
                        }
                    }
                    rustc_hir::QPath::TypeRelative(ty, path_segment) => {
                        println!("TypeRelative path: {:?}", path_segment);
                        let callee_def_id = path_segment.hir_id.owner.to_def_id();
                        println!("Callee def_id: {:?}", callee_def_id);
                        let new_call = Call {
                            call_expr: hir_id,
                            call_expr_span: expr.span,
                            caller: self.cur_fn,
                            caller_span: None,
                            callee: callee_def_id,
                            callee_span: path_segment.ident.span,
                            constraint_depth: self.constraint_depth,
                        };
                        let test_call = self.static_calls.get(&new_call).cloned();
                        println!("test_call: {:?}", test_call);
                
                            // 检查是否已经存在相同的调用（只比较 caller 和 callee）
                        if let Some(existing_call) = self.static_calls.get(&new_call).cloned() {
                            // 如果存在相同的 caller 和 callee，比较约束深度
                            if existing_call.should_insert(self.constraint_depth) {
                                // 移除现有的调用
                                self.static_calls.remove(&existing_call);
                                // 插入新的调用
                                self.static_calls.insert(new_call);
                                println!("Replaced existing call with new call, constraint_depth: {}", self.constraint_depth);
                            }
                        } else {
                            self.static_calls.insert(new_call);
                            println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                        }
                    }
                    rustc_hir::QPath::LangItem(_, span) => {
                        println!("LangItem path: {:?}", span); // 打印语言项路径信息
                    }
                }
            },
            rustc_hir::ExprKind::MethodCall(_, _, _, _) => {
                //println!("into method call, constraint_depth: {}", self.constraint_depth);
                let o_def_id = hir_id.owner;
                let typeck_tables = self.tcx.typeck(o_def_id);
                let substs = typeck_tables.node_args(hir_id);
                let method_id = typeck_tables.type_dependent_def_id(hir_id).expect("fail");
                let param_env = self.tcx.param_env(method_id);
                if let Ok(Some(inst)) =
                    self.tcx.resolve_instance_raw(ParamEnvAnd { param_env, value: (method_id, substs) })
                {
                    let res_def_id = inst.def_id();
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
                                constraint_depth: self.constraint_depth,
                            };

                            // 检查是否已经存在相同的动态调用
                            if let Some(existing_call) = self.dynamic_calls.get(&new_call).cloned() {
                                // 如果存在相同的 caller 和 callee，比较约束深度
                                if existing_call.should_insert(self.constraint_depth) {
                                    // 移除现有的调用
                                    self.dynamic_calls.remove(&existing_call);
                                    // 插入新的调用
                                    self.dynamic_calls.insert(new_call);
                                    println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                                }
                            } else {
                                self.dynamic_calls.insert(new_call);
                                println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                            }
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
                                constraint_depth: self.constraint_depth,
                            };

                            
                            // 检查是否已经存在相同的静态调用
                            if let Some(existing_call) = self.static_calls.get(&new_call).cloned() {
                                // 如果存在相同的 caller 和 callee，比较约束深度
                                if existing_call.should_insert(self.constraint_depth) {
                                    // 移除现有的调用
                                    self.static_calls.remove(&existing_call);
                                    // 插入新的调用
                                    self.static_calls.insert(new_call);
                                    println!("replaced new call, constraint_depth: {}", self.constraint_depth);
                                }
                            } else {
                                self.static_calls.insert(new_call);
                                println!("Inserted new call, constraint_depth: {}", self.constraint_depth);
                            }
                        }
                        None => (),
                        _ => todo!()
                    };
                }
            },
            _ => {
                //println!("Processing other expression: {:?}", expr);
            }
        }

        intravisit::walk_expr(self, expr); // 确保遍历所有表达式
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
