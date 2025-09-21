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
use crate::{routes::*, *};


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
    youtube_id: String,
    u: Option<String>
}


impl Video {
    pub fn new(id: i64, youtube_id: String, u: String) -> Self {
        Video { id: id, youtube_id: youtube_id, u: Some(u) }
    }

    pub fn remove_username(&mut self) {
        self.u = None;
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
struct OutgoingGetResponse { videos: Vec<Video>, c: i64, limit_reached: Vec<bool>, stage: i64, current_deadline: i64 }


pub fn handle_get(request: &Request, db: &mut Transaction, user: &User, state: &mut State) -> Response {
    let IncomingGetResponse { c } = try_or_400!(rouille::input::json_input(request));

    if !state.is_voting_allowed() { Response::message_json("Voting isn't allowed at this time").with_status_code(423); }
    if c < 0 || c > NUMBER_OF_CATEGORIES { Response::message_json("bad payload").with_status_code(400); }

    let mut limit_reached = state.get_voter_record(user.id);

    let mut outgoing = OutgoingGetResponse { 
        videos: vec![], 
        c, 
        limit_reached: limit_reached, 
        stage: state.current_stage(), 
        current_deadline: state.current_round_unix_deadline() 
    };

    match prep_votable_videos(&db, c, user.id, state.videos_per_vote(), state.do_include_usernames()) {
        DbResult::Ok(videos) => {
            outgoing.videos = videos;
            return Response::json(&outgoing);
        },
        DbResult::Err(error) => {
            println!("{:?}", error);
            return Response::message_json("Failed to fetch videos").with_status_code(500);
        },
        DbResult::Empty => {
            return Response::json(&outgoing).with_status_code(204);
        }
    }
}


// Picks videos at random that arn't in the banned or disqualified tables. Is tolerant to an inadequate number of videos
fn prep_votable_videos(db: &Transaction, mut category: i64, uid: i64, amount: i64, do_include_usernames: bool) -> DbResult<Vec<Video>, Error> {
    let videos;
    match || -> Result<Vec<Video>, Error> {
        let mut stmt = db.prepare(QUERY_GET_NEW_VOTABLE_VIDEOS)?;
        // If the category is 0, which signifies "any," modulate unix timestamp to pseudo-randomly choose a category 
        if category == 0 { category = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % (NUMBER_OF_CATEGORIES as u128)) as i64 + 1; }
        let mut rows = stmt.query([category.to_string(), amount.to_string()])?;
        let mut vids: Vec<Video> = Vec::new();
        while let Some(row) = rows.next()? {
            // Get username by default, remove it if state says no
            let mut video = Video::new(row.get(0)?, row.get(1)?, row.get(2)?);
            if !do_include_usernames { video.remove_username(); }
            vids.push(video);
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


pub fn handle_post(request: &Request, db: &mut Transaction, user: &User, state: &mut State) -> Response {
    // Parse request
    let IncomingPostRequest { incoming_list } = try_or_400!(rouille::input::json_input(request));

    if !state.is_voting_allowed() { Response::message_json("Voting isn't allowed at this time").with_status_code(423); }
    
    // Validate vote by comparing it against the user's active votes
    match || -> Result<Vec<(i64, i64, i64)>, Error> {
        let mut stmt = db.prepare(QUERY_GET_ACTIVE_VOTE_VIDEOS)?;
        let mut rows = stmt.query([user.id])?;
        let mut active_list: Vec<(i64, i64, i64)> = Vec::new();
        while let Some(row) = rows.next()? {
            active_list.push((row.get(0)?, row.get(1)?, row.get(1)?));
        }
        Ok(active_list)
    }() {
        Ok(active_list) => {
            // User voted too fast
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            if time < active_list[0].2 as u128 {
                return Response::message_json("You're voting too fast! Slow down!").with_status_code(429);
            }

            // Someone tried to vote before receiving any videos to vote on
            if active_list.len() == 0 {
                println!("Invalid vote attempt: User has no active votes, but attempted to submit something anyway: {:?}", incoming_list);
                return Response::message_json("Vote submitted").with_status_code(200);
            }

            // Video ids user submitted do not match the contents found in active votes
            for (id, _, _) in &active_list {
                if !incoming_list.contains(id) {
                    println!("Invalid vote attempt: Vote does not match user's active vote: Submitted: {:?}, expected {:?}", incoming_list, active_list);
                    return Response::message_json("Vote submitted").with_status_code(200);
                }
            }

            // User attempted to vote with too many or not enough videos
            if incoming_list.len() != active_list.len() {
                println!("Invalid vote attempt: Length does not match user's active_vote: Submitted: {}, expected: {}", incoming_list.len(), active_list.len());
                return Response::message_json("Vote submitted").with_status_code(200);
            }

            // User is banned
            if user.vote_banned {
                println!("Invalid vote attempt: This user has been shadow banned");
                return Response::message_json("Vote submitted").with_status_code(200);
            }
            
            let vote_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
            
            if let Err(_) = || -> Result<(), Error> {
                db.execute(QUERY_CLEAR_ACTIVE_VOTES, [user.id])?;
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
                    db.execute(QUERY_VOTE, [user.id, target, score, opponent, state.current_round(), vote_time])?;
                    
                    index += 1;
                }

                state.update_voter_record(user.id, active_list[0].1);

                Ok(())
            }() {
                return Response::message_json("Failed while casting vote").with_status_code(500);
            }
        },
        Err(_) => {
            return Response::message_json("Failed while trying to validate vote").with_status_code(500);
        }
    }
    
    Response::message_json("Vote submitted").with_status_code(200)
}