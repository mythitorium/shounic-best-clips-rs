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
// GET
//
//
//


#[derive(Deserialize)]
struct IncomingGetRequest {
    token: String,
}

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


#[derive(Deserialize)]
struct IncomingPostRequest {
    token: String,
    voting_round: Option<i64>,
    videos_per_vote: Option<i64>,
    unix_deadline: Option<i64>,
    limit_votes: Option<bool>,
}


pub fn handle_post(request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingPostRequest { 
        token, 
        voting_round, 
        videos_per_vote, 
        unix_deadline, 
        limit_votes
    } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }

    if let Some(voting_round) = voting_round       { state.set_voting_round(voting_round); }
    if let Some(videos_per_vote) = videos_per_vote { state.set_videos_per_vote(videos_per_vote); }
    if let Some(unix_deadline) = unix_deadline     { state.set_unix_deadline(unix_deadline); }
    if let Some(limit_votes) = limit_votes        { state.set_limit_votes(limit_votes); }

    Response::text("Server config updated").with_status_code(200)
}




