use parity_scale_codec::{Decode, Encode};
use sails_macros::query_handlers;
use scale_info::TypeInfo;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct DoThatParam {
    p1: u32,
    p2: String,
}

#[query_handlers]
mod queries {
    use super::*;

    fn this(p1: u32, p2: String) -> Result<(String, u32), u32> {
        Ok((p2, p1))
    }

    fn that(p1: DoThatParam) -> Result<(u32, String), String> {
        Ok((p1.p1, p1.p2))
    }
}

fn main() {
    let this_query = queries::Queries::This(1, "2".into());
    let _that_query = queries::Queries::That(DoThatParam {
        p1: 1,
        p2: "2".into(),
    });
    let (_response, _is_error): (queries::QueryResponses, bool) =
        queries::handlers::process_queries(this_query);
}
