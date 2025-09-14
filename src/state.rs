//
// state.rs
//
//
//

use std::{collections::HashMap, fs::{self, File}, hash::Hash};
use jwt_simple::{prelude::*, Error};

// This is the server state. It stores configuration and handles any caching that's best done outside a database
//
// It also creates and manages it's own file (currently set to config/config.toml), such that basic config can be cached between sessions

pub const NUMBER_OF_CATEGORIES: i64 = 2;
const CONFIG_FILENAME: &str = "server_config.toml";

pub struct State {
    config: Config,

    // These values are not
    voter_cache: HashMap<i64, Vec<bool>>,
    vote_total_cache: HashMap<i64, Tally>,
    token_cache: Vec<String>,
    jwt_key: HS256Key
}


#[derive(Serialize, Deserialize)]
pub struct Config {
    current_voting_round: i64,
    videos_per_vote: i64,
    current_round_unix_deadline: i64,
    vote_limiter_enabled: bool,
}


impl Config {
    pub fn new() -> Config {
        toml::from_str(&fs::read_to_string(CONFIG_FILENAME).unwrap_or(".".to_string()))
            .unwrap_or(Config { current_voting_round: 1, videos_per_vote: 2, current_round_unix_deadline: 0, vote_limiter_enabled: false })
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
            vote_total_cache: HashMap::new(),
            token_cache: Vec::new(),
            jwt_key: HS256Key::generate()
        }
    }

    pub fn tally_score(&mut self, video_id: i64, score: i64, round: i64) {
        self.vote_total_cache.entry(video_id).and_modify(|tally| tally.tally_score(score, round)).or_insert(Tally::new());
    }

    pub fn current_round(&self) -> i64 {
        self.config.current_voting_round
    } 

    pub fn videos_per_vote(&self) -> i64 {
        self.config.videos_per_vote
    }

    pub fn vote_limiter_enabled(&self) -> bool {
        self.config.vote_limiter_enabled
    }

    pub fn get_voter_record(&self, user_id: i64) -> Vec<bool> {
        if self.config.vote_limiter_enabled {
            return self.voter_cache.get(&user_id).unwrap_or(&vec![false, false]).clone();
        } else {
            return vec![false, false];
        }
        
    }

    pub fn update_voter_record(&mut self, user_id: i64, category: i64) {
        if self.config.vote_limiter_enabled && category < (NUMBER_OF_CATEGORIES + 1) && category > 0 {
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

    pub fn set_voting_round(&mut self, new_round: i64) {
        self.voter_cache.clear();
        self.config.current_voting_round = new_round;
        self.config.save();
    }

    pub fn set_videos_per_vote(&mut self, new_vote_size: i64) {
        self.config.videos_per_vote = new_vote_size;
        self.config.save();
    }

    pub fn set_vote_limiter(&mut self, limit_votes: bool) {
        self.config.vote_limiter_enabled = limit_votes;
        self.config.save();
    }

    pub fn set_unix_deadline(&mut self, new_deadline: i64) {
        self.config.current_round_unix_deadline = new_deadline;
        self.config.save();
    }

    pub fn current_round_unix_deadline(&self) -> i64 {
        self.config.current_round_unix_deadline
    }

    pub fn generate_new_token(&mut self) -> Result<String, Error> {
        let claims = Claims::create(Duration::from_mins(2));
        let token = self.jwt_key.authenticate(claims)?;
        self.token_cache.push(token.clone());
        Ok(token)
    }

    pub fn validate_token(&self, token: &String) -> bool {
        if let Ok(_) = self.jwt_key.verify_token::<NoCustomClaims>(token, None) {
            return true;
        } else {
            return false;
        }
    }

    pub fn save_self(&mut self) {
        const LOCATION: &str = "config/config.toml";
        // TODO: To this
    }

    pub fn config(&self) -> &Config {
        return &self.config;
    }
}


// This is a simple vote & winrate tracker which is used in the State's vote totals
// It's a simple abstraction that makes updating vote totals and getting a winrate for a given video more ergonomic 
pub struct Tally {
    scores: HashMap<i64, HashMap<i64, i64>>
}

impl Tally {
    pub fn new() -> Self {
        Tally { scores: HashMap::new() }
    }

    // Returns a number rounded to three decimal places
    pub fn ratio(&self, round: i64) -> f64 {
        let mut total_tally_count = 0;
        let mut total_score_amount = 0;
        for (score_value, tally_total) in { &self.scores[&round] }.into_iter() {
            total_tally_count += tally_total;
            total_score_amount += tally_total * score_value;
        }
        ((total_score_amount as f64 / total_tally_count as f64) * 100.).round() / 100.
        //((self.wins as f64 / (self.wins as f64 + self.losses as f64)) * 100. * 100.).round() / 100.
    }

    pub fn tally_score(&mut self, score: i64, round: i64) {
        *self.scores
            .entry(round)
            .or_insert(HashMap::new())
            .entry(score)
            .or_insert(0)
            += 1;
    }
}


struct VoteLimiter {
    pub enabled: bool,
    pub cache: HashMap<i64, Vec<bool>>
}


impl VoteLimiter {
    pub fn new() -> Self {
        VoteLimiter { enabled: false, cache: HashMap::new() }
    }
}
