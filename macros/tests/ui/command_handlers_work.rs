use parity_scale_codec::{Decode, Encode};
use sails_macros::command_handlers;
use scale_info::TypeInfo;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct DoThatParam {
    p1: u32,
    p2: String,
}

#[command_handlers]
mod commands {
    use super::*;

    fn do_this(p1: u32, p2: String) -> Result<(String, u32), u32> {
        Ok((p2, p1))
    }

    async fn do_that(p1: DoThatParam) -> Result<(u32, String), String> {
        Ok((p1.p1, p1.p2))
    }
}

#[tokio::main]
async fn main() {
    let do_this_cmd = commands::Commands::DoThis(1, "2".into());
    let _do_that_cmd = commands::Commands::DoThat(DoThatParam {
        p1: 1,
        p2: "2".into(),
    });
    let (_response, _is_error): (commands::CommandResponses, bool) =
        commands::handlers::handle_commands(do_this_cmd).await;
}
