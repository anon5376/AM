use am001::parser::rule::parse_rule_line;

#[test]
fn rule_parser_accepts_supported_commands_and_rejects_bad_input() {
    assert_eq!(parse_rule_line("", 1).unwrap().id, 1);
    assert_eq!(
        parse_rule_line("assert rust truth_assert=1 --weight 0.7", 2)
            .unwrap()
            .asserts
            .len(),
        1
    );
    assert_eq!(
        parse_rule_line("remember rust goal_relevance=0.8", 3)
            .unwrap()
            .asserts
            .len(),
        1
    );
    assert_eq!(parse_rule_line("cue rust 0.8", 4).unwrap().cues.len(), 1);
    assert_eq!(
        parse_rule_line("link rust am001 1", 5).unwrap().links.len(),
        1
    );
    assert_eq!(
        parse_rule_line("goal push rust", 6).unwrap().goal_ops.len(),
        1
    );
    assert!(parse_rule_line("assert rust certainty=1", 7).is_err());
    assert!(parse_rule_line("cue rust 2", 8).is_err());
    assert!(parse_rule_line("nonsense rust", 9).is_err());
}
