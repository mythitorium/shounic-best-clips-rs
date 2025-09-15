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
    videos_per_vote: Option<i64>,
    unix_deadline: Option<i64>,
    limit_votes: Option<bool>,
    elimination_threshold: Option<i64>,
    allow_voting: Option<bool>
}


pub fn handle_post(request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingPostRequest { 
        token, 
        videos_per_vote, 
        unix_deadline, 
        limit_votes,
        elimination_threshold,
        allow_voting
    } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }
    if let Some(new_videos_per_vote) = videos_per_vote { state.set_videos_per_vote(new_videos_per_vote); }
    if let Some(new_unix_deadline) = unix_deadline     { state.set_unix_deadline(new_unix_deadline); }
    if let Some(new_limit_votes) = limit_votes        { state.set_limit_votes(new_limit_votes); }
    if let Some(new_allow_voting) = allow_voting       { state.allow_voting(new_allow_voting); }
    if let Some(new_elimination_threshold) = elimination_threshold        { state.do_round_progression(new_elimination_threshold); }

    Response::text("Server config updated").with_status_code(200)
}




