//
// state.rs
//
//
//

use std::{collections::HashMap, fs::{self, File}, hash::Hash, time::{SystemTime, UNIX_EPOCH}};
use jwt_simple::{prelude::*, Error};
use rusqlite::Transaction;

use crate::sql::*;

// This is the server state. It stores configuration and handles any caching that's best done outside a database
//
// It also creates and manages it's own file (currently set to config/config.toml), such that basic config can be cached between sessions

pub const NUMBER_OF_CATEGORIES: i64 = 2;
const CONFIG_FILENAME: &str = "server_config.toml";
const ALLOW_VOTING_BY_DEFAULT: bool = true;

pub struct State {
    config: Config,

    // These values are not
    voter_cache: HashMap<i64, Vec<bool>>,
    jwt_key_pair: ES256KeyPair,
}


struct Tally { pub total_votes: i64, pub total_score: i64 }


#[derive(Serialize, Deserialize)]
pub struct Config {
    voting_round: i64,
    elimination_threshold: i64,
    videos_per_vote: i64,
    unix_deadline: i64,
    limit_votes: bool,
    allow_voting: bool
}


impl Config {
    pub fn new() -> Config {
        toml::from_str(&fs::read_to_string(CONFIG_FILENAME).unwrap_or(".".to_string()))
            .unwrap_or(Config { voting_round: 1, videos_per_vote: 2, unix_deadline: 0, limit_votes: false, elimination_threshold: 9999, allow_voting: ALLOW_VOTING_BY_DEFAULT })
    }

    pub fn save(&self) {
        fs::write(CONFIG_FILENAME, toml::to_string(self).unwrap());
    }
}


impl State {
    pub fn new() -> Self {
        State { 
            config: Config::new(),
            voter_cache: HashMap::new(),
            jwt_key_pair: ES256KeyPair::generate()
        }
    }
    
    pub fn current_round(&self) -> i64 {
        self.config.voting_round
    } 


    pub fn videos_per_vote(&self) -> i64 {
        self.config.videos_per_vote
    }

    
    pub fn limit_votes(&self) -> bool {
        self.config.limit_votes
    }


    pub fn current_round_unix_deadline(&self) -> i64 {
        self.config.unix_deadline
    }


    pub fn get_voter_record(&self, user_id: i64) -> Vec<bool> {
        if self.config.limit_votes {
            return self.voter_cache.get(&user_id).unwrap_or(&vec![false, false]).clone();
        } else {
            return vec![false, false];
        }
        
    }


    pub fn is_voting_allowed(&self) -> bool {
        return self.config.allow_voting;
    }


    pub fn update_voter_record(&mut self, user_id: i64, category: i64) {
        if self.config.limit_votes && category < (NUMBER_OF_CATEGORIES + 1) && category > 0 {
               self.voter_cache
                .entry(user_id)
                .and_modify(|votes| votes[category as usize - 1] = true )
                .or_insert_with(|| -> Vec<bool> { 
                    let mut d = vec![false, false]; 
                    d[category as usize - 1] = true; 
                    d 
                });
        }
    }
    

    pub fn set_videos_per_vote(&mut self, new_vote_size: i64) {
        self.config.videos_per_vote = new_vote_size;
        self.config.save();
    }


    pub fn set_limit_votes(&mut self, limit_votes: bool) {
        self.config.limit_votes = limit_votes;
        self.config.save();
    }

    
    pub fn set_unix_deadline(&mut self, new_deadline: i64) {
        self.config.unix_deadline = new_deadline;
        self.config.save();
    }


    pub fn allow_voting(&mut self, allow_voting: bool) {
        self.config.allow_voting = allow_voting;
    }


    pub fn do_round_progression(&mut self, db: &mut Transaction, new_elimination_threshold: i64) {
        // Apply/update parameters
        self.config.elimination_threshold = new_elimination_threshold;
        self.config.voting_round += 1;
        self.config.save();

        // Eliminate
        for i in 1..=NUMBER_OF_CATEGORIES {
            db.execute(QUERY_ELIMINATE_VIDEOS, [i, self.current_round(), i, new_elimination_threshold]);
        }

        self.voter_cache.clear();
    }


    //pub fn tally_vote(&mut self, id: i64, score: i64) {
    //    self.tally_cache
    //        .entry(id)
    //        .and_modify(|tally| { tally.total_votes += 1; tally.total_score += score; })
    //        .or_insert(Tally { total_score: score, total_votes: 1});
    //}


    pub fn eliminate_videos(&mut self, db: &mut Transaction, threshold: i64) {

    }


    // Create a new session token
    pub fn generate_new_token(&mut self) -> Result<String, Error> {
        let claims = Claims::create(Duration::from_secs(30));
        let token = self.jwt_key_pair.sign(claims)?;
        Ok(token)
    }


    // Validate a session token
    pub fn validate_token(&self, token: &String) -> bool {
        if let Ok(claims) = self.jwt_key_pair.public_key().verify_token::<NoCustomClaims>(token, None) {
            return claims.expires_at.unwrap().as_secs() > SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        } else {
            return false;
        }
    }

    pub fn config(&self) -> &Config {
        return &self.config;
    }
}
