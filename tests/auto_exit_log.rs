//! `auto_exit_log!` 用绝对路径引用 `orion_infra::logging::BoolFlag`，
//! 因此在 crate 内部无法展开；这里按外部使用者的方式来验证。

#[test]
fn auto_exit_log_takes_the_failure_branch_when_never_marked() {
    let recorded = std::cell::RefCell::new(Vec::<&'static str>::new());
    {
        let _guard = orion_infra::auto_exit_log!(
            recorded.borrow_mut().push("success"),
            recorded.borrow_mut().push("failure")
        );
    }

    assert_eq!(recorded.into_inner(), vec!["failure"]);
}
