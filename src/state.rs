//
// state.rs
//
//
//

use std::{collections::HashMap, hash::Hash};
use jwt_simple::{prelude::*, Error};

// This is the server state. It stores configuration and handles any caching that's best done outside a database
//
// It also creates and manages it's own file (currently set to config/config.toml), such that basic config can be cached between sessions
pub struct State {
    // These values are saved to a config file on disk
    current_voting_round: i64,
    videos_per_vote: i64,
    display_deadline_unix: usize,
    vote_limit: Option<i64>,

    // These values are not
    cached_vote_totals: HashMap<i64, Tally>,
    token_cache: Vec<String>,
    categories: Vec<Category>, // static
    jwt_key: HS256Key
}


impl State {
    pub fn new(unix: usize) -> Self {
        State { 
            current_voting_round: 1,
            videos_per_vote: 2,
            display_deadline_unix: unix,
            vote_limit: None,

            cached_vote_totals: HashMap::new(),
            categories: vec![Category::new("funny"), Category::new("skill")],
            token_cache: Vec::new(),
            jwt_key: HS256Key::generate()
        }
    }

    pub fn tally_score(&mut self, video_id: i64, score: i64, round: i64) {
        self.cached_vote_totals.entry(video_id).and_modify(|tally| tally.tally_score(score, round)).or_insert(Tally::new());
    }

    pub fn current_round(&self) -> i64 {
        self.current_voting_round
    } 

    pub fn videos_per_vote(&self) -> i64 {
        self.videos_per_vote
    }

    pub fn vote_limit(&self) -> i64 {
        self.vote_limit.unwrap_or(0)
    }

    pub fn time_left(&self) -> i64 {
        self.display_deadline_unix as i64
    }

    pub fn generate_new_token(&mut self) -> Result<String, Error> {
        let claims = Claims::create(Duration::from_hours(2));
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


struct Category {
    participation_threshold: i64,
    name: String
}


impl Category {
    pub fn new(name: &str) -> Self {
        Category { participation_threshold: 10000, name: name.to_string() }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_threshold(&mut self, new: i64) {
        self.participation_threshold = new;
    } 

    pub fn get_threshold(&self) -> i64 {
        self.participation_threshold
    }
}
