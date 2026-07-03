use crate::core::inspect::{dump_state, format_diff};
use crate::core::state::AmState;
use crate::core::step::step_result;
use crate::parser::rule::parse_rule_line;
use anyhow::Result;
use std::io::{self, Write};

pub fn run_repl(state: &mut AmState) -> Result<()> {
    let mut line = String::new();
    loop {
        print!("am> ");
        io::stdout().flush()?;
        line.clear();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }
        if trimmed == "dump" {
            print!("{}", dump_state(state, "act", 20)?);
            continue;
        }
        let event = parse_rule_line(trimmed, state.tick + 1)?;
        let trace = step_result(state, &event)?;
        print!("{}", format_diff(state, &trace));
    }
    Ok(())
}
