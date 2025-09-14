//
//
// GET, POST /server/tables
//
// 
//
//

use serde::{Deserialize, Serialize};
use rouille::Request;
use rusqlite::{Transaction};
use rouille::Response;
use crate::{state::State, User};


const ROWS_PER_PAGE: i64 = 200;


#[derive(Deserialize, Serialize)]
enum Table {
    Users = 0,
    Videos = 1,
    Reports = 2,
    Submissions = 3
}


#[derive(Deserialize, Serialize)]
struct IncomingGetRequest {
    token: String,
    page: i64,
    table: Table
}


#[derive(Deserialize, Serialize)]
enum Cell {
    Num(i64),
    String(String)
}


#[derive(Deserialize, Serialize)]
struct OutgoingGetResponse {
    rows: Vec<Vec<Cell>>
}


pub fn handle_get(_request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    Response::empty_404()
}


#[derive(Deserialize, Serialize)]
struct IncomingPostRequest {
    token: String,
    relevant_db_ids: Vec<i64>,
    table: Table,
    action_type: Action,
    action_outcome: bool
}


#[derive(Deserialize, Serialize)]
enum Action {
    VideoEliminate = 0,
    VideoDisqualify = 1,
    UserVoteBan = 2,
    UserReportBan = 3,
    ReportResolve = 4,
    SubmissionInclude = 5,
    SubmissionApprove = 6
}


pub fn handle_post(_request: &Request, _db: &mut Transaction, _user: &User, _state: &mut State) -> Response {
    Response::empty_404()
}