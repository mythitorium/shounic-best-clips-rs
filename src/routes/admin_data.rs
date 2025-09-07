//
//
// GET, POST /admin/data
//
// a json payload endpoint which acts as a dynamic way to request data from the server's sql tables and as a method of directly interacting with server data via webpage
//
//

use serde::{Deserialize, Serialize};
use rouille::Request;
use rusqlite::{Transaction};
use rouille::Response;
use crate::state::State;

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


#[derive(Serialize, Deserialize)]
struct Action {
    action_type: ActionType,
    display_text: String,
}


#[derive(Serialize, Deserialize)]
enum ActionType {
    // Row specific actions
    Delete,                       // Remove from a table
    Ban,                          // Add a clip to the banned table
    Disqualify,                   // Add a clip to the disqualified table
    Unban,                        // Remove a clip from the banned table

    // General actions
    MoveAllFromRawToClips,        // Self explanatory
    SetQualifyThreshold,           
    SetVoteLimit,
    SetPairSize,
}


#[derive(Deserialize)]
enum Table {
    Users,
    Clips,
    Disqualified,
    Banned,
    Reports,
    Raw
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
    desired_table: Table,     // source the data should pull from
    page: i32,                // table page. Page size is 200 btw
}


#[derive(Serialize)]
struct OutgoingGetResponse {
    general_actions: Vec<Action>,
    item_actions: Vec<Action>,
    columns: Vec<String>,
    rows: Vec<Vec<TableCell>>,
    page: i32
}


pub fn handle_get(_request: &Request, _db: &mut Transaction, _uid: i64, _state: &mut State) -> Response {
    Response::empty_404()
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
    target_table: Option<Table>,
    target_row_id: i32,
    action: ActionType
}


pub fn handle_post(_request: &Request, _db: &mut Transaction, _uid: i64, _state: &mut State) -> Response {
    Response::empty_404()
}




