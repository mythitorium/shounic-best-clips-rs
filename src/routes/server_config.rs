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
use crate::{routes::*, *};


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
    let IncomingGetRequest { token} = try_or_400!(serde_qs::from_str::<IncomingGetRequest>(request.raw_query_string()));

    if !state.validate_token(&token) { return Response::message_json("bad credentials").with_status_code(401); }

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
    voting_stage: Option<i64>,
    videos_per_vote: Option<i64>,
    unix_deadline: Option<i64>,
    limit_votes: Option<bool>,
    elimination_threshold: Option<i64>,
    allow_voting: Option<bool>,
    include_usernames: Option<bool>
}


pub fn handle_post(request: &Request, db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingPostRequest { 
        token, 
        voting_stage,
        videos_per_vote, 
        unix_deadline, 
        limit_votes,
        elimination_threshold,
        allow_voting,
        include_usernames
    } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::message_json("bad credentials").with_status_code(401); }

    if let Some(new_videos_per_vote) = videos_per_vote                    { state.set_videos_per_vote(new_videos_per_vote); }
    if let Some(new_voting_stage) = voting_stage                          { state.set_voting_stage(new_voting_stage); }
    if let Some(new_unix_deadline) = unix_deadline                        { state.set_unix_deadline(new_unix_deadline); }
    if let Some(new_limit_votes) = limit_votes                           { state.set_limit_votes(new_limit_votes); }
    if let Some(new_allow_voting) = allow_voting                         { state.allow_voting(new_allow_voting); }
    if let Some(new_elimination_threshold) = elimination_threshold        { state.do_round_progression(db, new_elimination_threshold); }
    if let Some(new_include_usernames) = include_usernames               { state.set_include_usernames(new_include_usernames); }

    Response::message_json("Server state updated").with_status_code(200)
}




