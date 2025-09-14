//
//
// GET /admin/login
//
// Login endpoint, turns user and password into a temp token which is used to validate all /admin/data requests
//
//

use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rouille::{try_or_400, Request};
use rusqlite::{Error, Transaction};
use rouille::Response;
use crate::{sql::QUERY_GET_USER_HASH, state::State, User};

//
//
//
// GET
//
//
//

#[derive(Deserialize, Debug)]
struct IncomingGetRequest { 
    username: String, 
    password: String 
}


#[derive(Serialize)]
struct OutgoingGetResponse { 
    token: String 
}


pub fn handle_get(request: &Request, db: &mut Transaction, _user: &User, state: &mut State) -> Response {
        let IncomingGetRequest { username, password } = try_or_400!(rouille::input::json_input(request));
        
        match || -> Result<String, Error> {
            db.query_row(QUERY_GET_USER_HASH, [username], |row| 
                Ok(row.get(0)?)
            )
        }() {
            Ok(password_hash) => {
                if let Ok(parsed_hash) = PasswordHash::new(&password_hash) {
                    if Argon2::default().verify_password(&password.into_bytes(), &parsed_hash).is_ok() {
                        if let Ok(token) = state.generate_new_token() {
                            return Response::json(&OutgoingGetResponse { token: token });
                        } else {
                            return Response::text("Internal auth failure").with_status_code(500);
                        }
                    } else {
                        return Response::text("Incorrect password").with_status_code(401);
                    }
                }
            },
            Err(error) => {
                match error {
                    Error::QueryReturnedNoRows => { return Response::text("User does not exist").with_status_code(401); },
                    _ => { return Response::text("Internal database failure").with_status_code(500); }
                }
            }
        }
    
        Response::text("Internal failure").with_status_code(500)
}