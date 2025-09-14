//
//
// GET, POST /server/config
//
// for getting/setting server config aka the 'parameters,' such as voting round and voting limits
//
//

use serde::{de::Visitor, Deserialize, Serialize};
use rouille::{try_or_400, Request};
use rusqlite::{Transaction};
use rouille::Response;
use crate::{state::State, User};

//
//
// 
// Internal Types
// 
//
//

#[derive(Serialize)]
enum TableCell {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool)
}


#[derive(Deserialize)]
enum Table {
    Users,
    Videos,
    Reports,
    Submissions
}

//
//
//
// GET
//
//
//

#[derive(Deserialize)]
struct IncomingGetRequest {
    token: String,
}


// Better idea: Just use the parameters instance using used by the state

//#[derive(Serialize)]
//struct OutgoingGetResponse {
//    current_voting_round: i64,
//    videos_per_vote: i64,
//    current_round_unix_deadline: i64,
//    vote_limiter_enabled: bool,
//}


pub fn handle_get(request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingGetRequest { token} = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }

    Response::json(state.config())
}

//
//
//
// POST
//
//
//

// TODO: Refactor to allow for carrying general action action details
#[derive(Deserialize)]
struct IncomingPostRequest {
    token: String,
    current_voting_round: Option<i64>,
    videos_per_vote: Option<i64>,
    current_round_unix_deadline: Option<i64>,
    vote_limiter_enabled: Option<bool>,
}


pub fn handle_post(request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingPostRequest { 
        token, 
        current_voting_round, 
        videos_per_vote, 
        current_round_unix_deadline, 
        vote_limiter_enabled
    } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }

    if let Some(current_voting_round) = current_voting_round { state.set_voting_round(current_voting_round); }
    if let Some(videos_per_vote) = videos_per_vote { state.set_videos_per_vote(videos_per_vote); }
    if let Some(current_round_unix_deadline) = current_round_unix_deadline { state.set_unix_deadline(current_round_unix_deadline); }
    if let Some(vote_limiter_enabled) = vote_limiter_enabled { state.set_vote_limiter(vote_limiter_enabled); }

    Response::text("Server config updated").with_status_code(200)
}




