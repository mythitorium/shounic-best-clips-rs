//
//
// GET, POST /vote
//
// the voting endpoint. GET sends videos, and POST commits incoming videos (referred to as votes, 
// however the literal data being sent back is just the GET response but ordered) to the database 
//
//

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rouille::{try_or_400, Request};
use rusqlite::{fallible_streaming_iterator::FallibleStreamingIterator, Error, Transaction};
use rouille::Response;
use serde::{Deserialize, Serialize};
use crate::{sql::*, state::State};

//
//
//
// Internal Types
//
//
//

enum DbResult<T, E> {
    Ok(T),
    Err(E),
    Empty
}

impl<T, E> From<Result<T, E>> for DbResult<T, E> {
    fn from(item: Result<T, E>) -> Self {
        match item {
            Ok(ok) => DbResult::Ok(ok),
            Err(err) => DbResult::Err(err)
        }
    }
}


#[derive(Serialize, Debug)]
struct Video {
    id: i64,
    youtube_id: String
}

impl Video {
    pub fn new(id: i64, youtube_id: String) -> Self {
        Video { id: id, youtube_id: youtube_id }
    }
}

//
//
//
// GET
//
//
//

#[derive(Deserialize)]
struct IncomingGetResponse { c: i64 }

#[derive(Serialize, Debug)]
struct OutgoingGetResponse { videos: Vec<Video>, c: i64, limit_reached: Vec<bool>, round: i64, current_deadline: i64 }

pub fn handle_get(request: &Request, db: &mut Transaction, uid: i64, state: &mut State) -> Response {
    let IncomingGetResponse { c } = try_or_400!(rouille::input::json_input(request));

    let mut limit_reached = state.get_voter_record(uid);

    let mut outgoing = OutgoingGetResponse { 
        videos: vec![], 
        c, 
        limit_reached: limit_reached, 
        round: state.current_round(), 
        current_deadline: state.current_round_unix_deadline() 
    };

    match prep_votable_videos(&db, c, uid, state.videos_per_vote()) {
        DbResult::Ok(videos) => {
            outgoing.videos = videos;
            return Response::json(&outgoing);
        },
        DbResult::Err(error) => {
            println!("{:?}", error);
            return Response::text("Failed to fetch videos").with_status_code(500);
        },
        DbResult::Empty => {
            return Response::json(&outgoing).with_status_code(204);
        }
    }
}

// Picks videos at random that arn't in the banned or disqualified tables. Is tolerant to an inadequate number of videos
fn prep_votable_videos(db: &Transaction, mut category: i64, uid: i64, amount: i64) -> DbResult<Vec<Video>, Error> {
    let videos;
    match || -> Result<Vec<Video>, Error> {
        let mut stmt = db.prepare(QUERY_GET_NEW_VOTABLE_VIDEOS)?;
        // If the category is 0, which signifies "any," modulate unix timestamp to semi randomly chose a category 
        if category == 0 { category = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(1)).as_millis() % 2) as i64 + 1; }
        let mut rows = stmt.query([category.to_string(), amount.to_string()])?;
        let mut vids: Vec<Video> = Vec::new();
        while let Some(row) = rows.next()? {
            vids.push(Video::new(row.get(0)?, row.get(1)?));
        }
        Ok(vids)
    }() {
        Ok(v) => {
            videos = v;
        }, 
        Err(error) => {
            return DbResult::Err(error);
        }
    }

    // Not enough videos 
    if (videos.len() as i64) < amount {
        return DbResult::Empty;
    }

    if let Err(err) = db.execute(QUERY_CLEAR_ACTIVE_VOTES, [uid]) {
        return DbResult::Err(err);
    }

    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    for video in &videos {
        if let Err(err) = db.execute(QUERY_SET_ACTIVE_VOTE, [uid.to_string(), video.id.to_string(), start_time.to_string(), category.to_string()]) {
            return DbResult::Err(err);
        }
    }

    // I should be using params for this, but this method is easier
    //let mut query = QUERY_SET_ACTIVE_VOTE_VALUELESS.to_string();
    //let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    //for video in &videos {
    //    query.push_str(format!("({}, {}, {}),", uid, video.id, start_time).as_str());
    //}
    //query.pop(); // remove the comma from the last insert item
    //query.push_str(";"); // idk, just feels right to do
    //if let Err(err) = db.execute(&query, []) {
    //    return DbResult::Err(err);
    //}

    return DbResult::Ok(videos);
}

//
//
//
// POST
//
//
//

#[derive(Deserialize)]
struct IncomingPostRequest { incoming_list: Vec<i64> }


pub fn handle_post(request: &Request, db: &mut Transaction, uid: i64, state: &mut State) -> Response {
    // Parse request
    let IncomingPostRequest { incoming_list } = try_or_400!(rouille::input::json_input(request));
    
    // Validate vote by comparing it against the user's active votes
    match || -> Result<Vec<(i64, i64)>, Error> {
        let mut stmt = db.prepare(QUERY_GET_ACTIVE_VOTE_VIDEOS)?;
        let mut rows = stmt.query([uid])?;
        let mut active_list: Vec<(i64, i64)> = Vec::new();
        while let Some(row) = rows.next()? {
            active_list.push((row.get(0)?, row.get(1)?));
        }
        Ok(active_list)
    }() {
        Ok(active_list) => {
            //Validation
            if active_list.len() == 0 {
                println!("Invalid vote attempt: User has no active votes, but attempted to submit something anyway: {:?}", incoming_list);
                return Response::text("Vote submitted").with_status_code(200);
            }
            for (id, _) in &active_list {
                if !incoming_list.contains(id) {
                    println!("Invalid vote attempt: Vote does not match user's active vote: Submitted: {:?}, expected {:?}", incoming_list, active_list);
                    return Response::text("Vote submitted").with_status_code(200);
                }
            }
            if incoming_list.len() != active_list.len() {
                println!("Invalid vote attempt: Length does not match user's active_vote: Submitted: {}, expected: {}", incoming_list.len(), active_list.len());
                return Response::text("Vote submitted").with_status_code(200);
            }
            
            let vote_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
            
            if let Err(_) = || -> Result<(), Error> {
                db.execute(QUERY_CLEAR_ACTIVE_VOTES, [uid])?;
                let mut index = 0;
                for id in &incoming_list {
                    let target = id.clone();
                    let opponent;
                    // if index is 0, opponent id is index 1. otherwise, opponent id is index 0.
                    //
                    // This does not work if the amount of incoming video ids is greater than 2. 
                    // However, this is designed to allow for vote purging disqualified videos, something which typically wouldn't happen in a final round
                    if index == 0 {
                        opponent = incoming_list[1];
                    } else {
                        opponent = incoming_list[0];
                    }
                    let score = incoming_list.len() as i64 - 1 - index;
                    db.execute(QUERY_VOTE, [uid, target, score, opponent, state.current_round(), vote_time])?;
                    state.tally_score(id.clone(), score, state.current_round());
                    
                    index += 1;
                }

                state.update_voter_record(uid, active_list[0].1);

                Ok(())
            }() {
                return Response::text("Failed while casting vote").with_status_code(500);
            }
        },
        Err(_) => {
            return Response::text("Failed while trying to validate vote").with_status_code(500);
        }
    }
    
    Response::text("Vote submitted").with_status_code(200)
}