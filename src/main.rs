//
//               _   _    
//     _ __ _  _| |_| |_  
//    | '  \ || |  _| ' \ 
//    |_|_|_\_, |\__|_||_|
//          |__/          
//
//

// TODO: Explain the tally vote system
// TODO: Make votes tally on vote submit

#![allow(warnings)]

mod sql;
mod state;
mod routes;

use sql::*;
use state::*;

use rouille::{self, router};
use std::error::Error as StdError;
//use serde;
//use serde_derive;
//use serde_derive::Serialize;
use std::{fs::{self, File}, net::SocketAddr, sync::Mutex, thread, time::{Duration, Instant} };

//use postgres::{Client, NoTls, Transaction};
use rusqlite::{Connection, Error, Result, Transaction};
use rouille::{Request, Response};


// Check out this cool bridge  |
//                             V
//                                           ---_
//                                .   .  . . o--o
// ============================================================================
//   /|\        \|/         /|\        \|/         /|\        \|/         /|\  
//  / | \        |         / | \        |         / | \        |         / | \ 


fn main() {
    let _ = fs::create_dir("db");
    let db = {
        let db = Connection::open("db/votes.db");
        Mutex::new(db.expect("Failed to connect to database"))
    };

    let state = Mutex::new(State::new()) ;

    // Initialization tasks
    {
        // Prep mutexes
        let db = db.lock().unwrap(); 
        let mut state = state.lock().unwrap();

        // Setup database tables and pragmas
        db.execute_batch(QUERY_SETUP).expect("Failed to initialize database");

        // Generate voter record cache
        match build_voter_record(&db, &mut state) {
            Ok(_) => {
                println!("Voter records loaded. Process took 0 seconds");
            },
            Err(error) => {
                println!("Failed to build voter record on initialization");
                println!("Reason: {:?}", error);
                return;
            }
        }

        // Cache vote totals to the state
        match tally_votes(&db, &mut state) {
            Ok(_) => {
                println!("Vote tallies loaded. Process took 0 seconds");
            },
            Err(error) => {
                println!("Failed to tally votes on initialization");
                println!("Reason: {:?}", error);
                return;
            }
        }
    }

    // Spawn coroutine thread
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(5000));
        while true {
            let _ = reqwest::blocking::get("http://localhost:8080/coroutine").unwrap().text().unwrap();
            thread::sleep(Duration::from_millis(60000));
        }
    });

    println!("");

    // Start server
    // Runs per request
    rouille::start_server("0.0.0.0:8080", move |request| {
        let start = std::time::Instant::now();

        // Prep mutexes
        let mut db = db.lock().unwrap();
        let mut db = db.transaction().unwrap();
        let mut state = state.lock().unwrap();

        // Get logging details ahead of time as `request` gets consumed in the upcoming switch block
        let method = request.method().to_string();
        let path = request.url();
        let cached_ip;

        // Store & process IP then handle routes
        let response: Response;
        match handle_ip(&request, &mut db) {
            Ok((uid, ip)) => {
                // Take ip outside of scope so it can be printed to terminal
                cached_ip = ip;
                
                // Handle route
                response = router!(request,
                    // Html
                    (GET)  (/) =>            { Response::from_file("text/html", File::open("www/index.html").unwrap()) },
                    (GET)  (/admin) =>       { Response::from_file("text/html", File::open("www/admin.html").unwrap()) }, 

                    // Json payload
                    (GET)  (/vote) =>        { routes::vote::handle_get(request, &mut db, uid, &mut state) }, 
                    (POST) (/vote) =>        { routes::vote::handle_post(request, &mut db, uid, &mut state) }, 
                    (GET)  (/admin/login) => { routes::login::handle_get(request, &mut db, uid, &mut state) }, 
                    (GET)  (/admin/data) =>  { routes::admin_data::handle_get(request, &mut db, uid, &mut state) }, 
                    (POST) (/admin/data) =>  { routes::admin_data::handle_post(request, &mut db, uid, &mut state) }, 

                    // Fine you stupid fucks. you can have your css files. mfs.
                    _ => rouille::match_assets(request, "www")
                );
            },
            Err(ip) => {
                cached_ip = ip;
                response = Response::text("Error while handling IP").with_status_code(500);
            }
        }

        //println!("\nTime elapsed after ip log and request process: {:?}", start.elapsed());

        // Commit database transaction
        if response.is_success() {
            db.commit().unwrap();
        }

        //println!("Time elapsed after transaction commit: {:?}", start.elapsed());

        // Log response details to terminal
        log_outgoing(cached_ip, start, &response, path, method);

        response
    });
}


// This
// 1. separates port from id.
// 2. Log IP to the database and returns the user ID as it exists in the db.
// returns Result<(db user id, ip)>, ip>
fn handle_ip(req: &Request, db: &mut Transaction) -> Result<(i64, String), String> {
    let address: &SocketAddr = req.remote_addr();
    let ip_string = address.ip().to_string();

    if let Err(_) = db.execute(QUERY_LOG_USER, [ip_string.clone()]) {
        return Err(ip_string);
    }
    if let Ok(uid) = db.query_row(QUERY_GET_USER_ID, [ip_string.clone()], |row| row.get(0)) {
        return Ok((uid, ip_string));
    } else {
        return Err(ip_string);
    }
}

// Logging
fn log_outgoing(ip: String, start_time: Instant, response: &Response, path: String, method: String) {
    let time_elapsed_string = format!("{}", format!("{:?}", start_time.elapsed()));
    println!("[{}] {}{} {} {} {} {}", 
        ip,
        time_elapsed_string,
        " ".repeat(12 - time_elapsed_string.chars().count()), 
        response.status_code,
        method,
        " ".repeat(4 - method.len()),
        path
    );
}

// Tally votes
// TODO: Explain what this does in detail
fn tally_votes(db : &Connection, state: &mut State) -> Result<(), Error> {
    let mut stmt = db.prepare(QUERY_GET_VOTES)?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

    for row in rows {
        let (video_id, score, round) = row?;
        state.tally_score(video_id, score, round);
    }

    Ok(())
}


// Build voter cache
fn build_voter_record(db : &Connection, state: &mut State) -> Result<(), Error> {
    let mut stmt = db.prepare(QUERY_GET_VOTES_THIS_ROUND)?;
    let rows = stmt.query_map([state.current_round()], |row| Ok((row.get(0)?, row.get(1)?)))?;

    for row in rows {
        let (user_id, category) = row?;
        state.update_voter_record(user_id, category);
    }

    Ok(())
}