//
//
// GET /metadata
//
// Returns a payload containing server metadata, such as voting time remaining and videos available to vote on.
// 
//

use rouille::Request;
use rusqlite::{Transaction};
use rouille::Response;
use serde::Serialize;
use crate::state::State;

//
//
//
// GET
//
//
//

#[derive(Serialize)]
struct OutgoingGetResponse {
    time_left: i64,
    vote_limit: i64,
    round: i64
}

pub fn handle_get(_request: &Request, _db: &mut Transaction, _uid: i64, state: &mut State) -> Response {
    return Response::json(&OutgoingGetResponse {
        time_left: state.time_left(),
        vote_limit: state.vote_limit(),
        round: state.current_round()
    });
}