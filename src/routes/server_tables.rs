//
//
// GET, POST /server/tables
//
// 
//
//

use serde::{Deserialize, Serialize};
use rouille::{try_or_400, Request};
use rusqlite::{Rows, Transaction, Error};
use rouille::Response;
use crate::{sql::{QUERY_FRONTEND_GET_RANKING_DATA, QUERY_FRONTEND_GET_REPORT_DATA, QUERY_FRONTEND_GET_USER_DATA, QUERY_FRONTEND_GET_VIDEO_DATA}, state::State, User};


//
//
//
// GET
//
//
//


#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum Table {
    Users = 0,
    Videos = 1,
    Rankings = 2,
    Reports = 3
}


#[derive(Deserialize, Serialize)]
struct IncomingGetRequest {
    token: String,
    page: i64,
    table: Table,
    round: i64,
    category: i64
}


#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum Cell {
    Num(i64),
    String(String)
}


#[derive(Deserialize, Serialize)]
struct OutgoingGetResponse {
    rows: Vec<Vec<Cell>>
}


const TABLE_CELL_AMOUNTS: [usize; 4] = [4, 5, 7, 7];
const ROWS_PER_PAGE: i64 = 200;


pub fn handle_get(request: &Request, db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingGetRequest { token, page, table, round, category } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }

    let mut real_rows: Vec<Vec<String>> = Vec::new();

    // Prepare statement
    let mut stmt= db.prepare({
        match table {
            Table::Users => QUERY_FRONTEND_GET_USER_DATA,
            Table::Videos => QUERY_FRONTEND_GET_VIDEO_DATA,
            Table::Rankings => QUERY_FRONTEND_GET_RANKING_DATA,
            Table::Reports => QUERY_FRONTEND_GET_REPORT_DATA,
        }
    }).unwrap();

    // Apply params and execute
    let mut rows = match table {
        Table::Users => stmt.query([page.to_string(), ((page-1)*ROWS_PER_PAGE).to_string()]).unwrap(),
        Table::Videos => stmt.query([page.to_string(), ((page-1)*ROWS_PER_PAGE).to_string()]).unwrap(),
        Table::Rankings => stmt.query([round.to_string(), category.to_string(), page.to_string(), ((page-1)*ROWS_PER_PAGE).to_string()]).unwrap(),
        Table::Reports => stmt.query([page.to_string(), ((page-1)*ROWS_PER_PAGE).to_string()]).unwrap()
    };

    // Process data. Turn everything into a string who fucking cares
    while let Some(row) = rows.next().unwrap_or(None) { 
        let mut real_row: Vec<String> = Vec::new();
            for i in 0..TABLE_CELL_AMOUNTS[table.clone() as usize] {
                real_row.push(row.get(i).unwrap_or("".to_string()));
            }
        real_rows.push(real_row); 
    };

    return Response::json(&real_rows).with_status_code(200);

    Response::empty_404()
}


//
//
//
// POST
//
//
//


#[derive(Deserialize, Serialize)]
struct IncomingPostRequest {
    token: String,
    target_id: i64,
    table: Table,
    action_type: Action,
    action_outcome: bool
}


#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum Action {
    VideoDisqualify = 0,
    UserVoteBan = 1,
    UserReportBan = 2,
    ReportResolve = 3
}


pub fn handle_post(request: &Request, _db: &mut Transaction, _user: &User, state: &mut State) -> Response {
    let IncomingPostRequest { token, target_id, table, action_type, action_outcome } = try_or_400!(rouille::input::json_input(request));

    if !state.validate_token(&token) { return Response::text("bad credentials").with_status_code(401); }

    match action_type {
        Action::VideoDisqualify => {

        },
        Action::UserVoteBan => {

        },
        Action::UserReportBan => {

        },
        Action::ReportResolve => {

        }
    }


    Response::empty_404()
}