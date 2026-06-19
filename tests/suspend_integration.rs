mod common;
use common::*;

#[test]
fn suspend_then_resume_toggles_prefix_and_keytable() {
    if !has_tmux() {
        eprintln!("skip");
        return;
    }
    let s = Server::start("suspend");
    let before = s.opt("prefix");

    assert!(s.huma(&["suspend"]).status.success());
    // tmux echoes the disabled prefix key back as "None".
    assert!(
        s.opt("prefix").eq_ignore_ascii_case("none"),
        "prefix should be disabled, got {:?}",
        s.opt("prefix")
    );
    assert_eq!(s.opt("key-table"), "suspended", "key-table should switch");

    assert!(s.huma(&["resume"]).status.success());
    assert_eq!(s.opt("prefix"), before, "prefix should be restored");
    assert_ne!(s.opt("key-table"), "suspended", "key-table should reset");
}
