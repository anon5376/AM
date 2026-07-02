use am001::world::action::Action;
use am001::world::script::parse_script;

#[test]
fn script_parsing_maps_to_closed_action_enum_and_rejects_unknowns() {
    let actions = parse_script("N S E W PickUp Drop Open Wait\n").unwrap();
    assert_eq!(actions, Action::tie_break_order());

    let err = parse_script("N Jump\n").unwrap_err();
    assert!(err.to_string().contains("parse action"));
}
