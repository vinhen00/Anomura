Performing manual macro expansion inside the after_crate_root_parsing callback allows you to mutate an AST (&mut ast::Crate) both before and after expansion. However, doing this involves several important technical considerations:

    Compilation Phase & State Availability
    Macro expansion normally depends on early name resolution and module discovery. At after_crate_root_parsing, the compiler has only parsed the top-level token stream; the default Resolver and macro expansion infrastructure have not been initialized yet.

    Manual Driver Setup
    To expand macros at this point, you must manually construct and drive internal rustc components (such as rustc_interface::create_resolver and rustc_expand::expand::MacroExpander). Once expanded, you must finalize the resolution outputs so downstream compiler passes receive a complete state.

    Compiler Internal APIs (rustc_private)
    This technique relies directly on unstable compiler internals (rustc_expand, rustc_session, rustc_interface, rustc_ast). These APIs change frequently across nightly Rust compiler versions, requiring high maintenance for plugin code.

    AST Node ID Invalidation and Renumbering
    Adding, removing, or duplicating AST nodes during or after expansion can disrupt ast::NodeId uniqueness and contiguity. You must allocate fresh IDs using ast_node_id_allocator or run a renumbering pass (rustc_ast_lowering::renumber) across the AST to prevent crashes during downstream HIR lowering.

    AST Validation
    Because you bypass standard phase ordering, running rustc_ast_passes::ast_validation::check_crate after your transformations is essential to confirm the AST remains valid and that no duplicate or unallocated node IDs were introduced.

    Immutability vs. Mutation Tradeoff
    While moving to the later after_expansion callback gives access to the compiler's official expanded AST, that AST is stored behind immutable context references (tcx) and cannot be mutated without runtime panics. Manual expansion inside after_crate_root_parsing remains the standard workaround when direct &mut ast::Crate access is required.




    Simple example idea: 

    Rust


impl rustc_driver::Callbacks for CrateIntercept {
    fn after_crate_root_parsing<'tcx>(
        &mut self,
        compiler: &Compiler,
        krate: &mut Crate,
    ) -> Compilation {
        let sess = &compiler.session();

        // 1. Run your initial AST mutations on unexpanded AST
        self.pre_expansion_ast_pass(krate);

        // 2. Programmatically expand macros in-place
        expand_crate_ast(sess, krate);

        // 3. Run post-expansion AST mutations with direct &mut Crate access
        self.post_expansion_ast_pass(krate);

        Compilation::Continue
    }
}

fn expand_crate_ast(sess: &Session, krate: &mut Crate) {
    // Initialize resolver and macro expansion infrastructure
    let mut resolver = rustc_interface::create_resolver(sess, krate);
    let mut expander = rustc_expand::expand::MacroExpander::new(
        sess,
        false, // ecfg.is_test
    );

    // Perform full recursive macro expansion on the AST root
    expander.expand_crate(krate, &mut resolver);

    // Finalize name resolution artifacts generated during expansion
    resolver.into_outputs();
}

impl MacroExpandingCallback {
    fn pre_expansion_ast_pass(&mut self, krate: &mut Crate) {
        // Your existing logic operating on &mut ast::Crate before expansion
    }

    fn post_expansion_ast_pass(&mut self, krate: &mut Crate) {
        // Your logic operating on fully expanded &mut ast::Crate
    }
}