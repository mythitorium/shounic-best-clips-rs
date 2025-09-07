//
// sql.rs
//
// Defining the sql queries and schema is so much easier in it's own file
//
// Param descriptions are for debugging/documentation
//

// ?1 - ip address
pub const QUERY_LOG_USER: &str = "INSERT OR IGNORE INTO users(ip) VALUES (?1)";

// ?1 - ip address
pub const QUERY_GET_USER_ID: &str = "SELECT id FROM users WHERE ip = ?1";

// ?1 - ip address
pub const QUERY_LOG_USER_RETURN_UID: &str = "INSERT OR IGNORE INTO users(ip) VALUES (?1) RETURNING id;";

// ?1 - category being filtered for
// ?2 - the amount
pub const QUERY_GET_NEW_VOTABLE_VIDEOS: &str = "
    SELECT id, youtube_id FROM  videos
        WHERE id NOT IN (SELECT id FROM culled_videos WHERE videos.id = culled_videos.video_id) 
        AND id NOT IN (SELECT id FROM disqualified_videos WHERE videos.id = disqualified_videos.video_id) 
        AND category = ?1
        ORDER BY random() LIMIT ?2
";

// ?1 - amount to get
pub const QUERY_GET_NEW_VOTABLE_VIDEOS_NO_CATEGORY: &str = "
    SELECT id, youtube_id FROM  videos
        WHERE id NOT IN (SELECT id FROM culled_videos WHERE videos.id = culled_videos.video_id) 
        AND id NOT IN (SELECT id FROM disqualified_videos WHERE videos.id = disqualified_videos.video_id)
        ORDER BY random() LIMIT ?1
";

// ?1 - user id
// ?2 - video 1
// ?3 - start time in unix
// ?4 - user id
// ?5 - video 2
// ?6 - start time in unix
pub const QUERY_SET_ACTIVE_2_VOTE: &str = "INSERT INTO active_votes(user_id, video_id, start_time) VALUES (?1, ?2, ?3), (?4, ?5, ?6)";

// same as 'QUERY_SET_ACTIVE_2_VOTE' but it's set up to be dynamically modified depending on the amount of active votes being casted
pub const QUERY_SET_ACTIVE_VOTE_VALUELESS: &str = "INSERT INTO active_votes(user_id, video_id, start_time) VALUES "; // `+ '({id}, {vid1}, {vid2}, {time})'`, for example


// ?1 - user id
// ?2 - winning video id
// ?3 - score
// ?4 - opponent_video_id
// ?5 - voting round
// ?6 - vote time in unix
pub const QUERY_VOTE: &str = "INSERT INTO votes(user_id, video_id, score, opponent_video_id, round, vote_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

// ?1 - user id
pub const QUERY_CLEAR_ACTIVE_VOTES: &str = "DELETE FROM active_votes WHERE user_id = ?1";

// ?1 - user id
pub const QUERY_GET_ACTIVE_VOTE_VIDEOS: &str = "SELECT id FROM videos JOIN active_votes ON active_votes.video_id = videos.id WHERE active_votes.user_id = ?1;";

// ?1 - video id
// ?2 - video rank after vote count
pub const QUERY_SET_FINALIST: &str = "
    INSERT INTO finalist_videos(video_id, rank) VALUES (?1, ?2); 
";

// ?1 - user id
// ?2 - video id
// ?3 - vote time
// ?4 - placement in final ranking vote
pub const QUERY_VOTE_FINALIST: &str = "
    DELETE FROM active_votes WHERE user_id = ?1;
    INSERT INTO finalist_votes(user_id, video_id, vote_time, rank) VALUES (?1, ?2, ?3, ?4); 
";

// TODO: Do QUERY_VOTE_FINALIST_VALUELESS? Maybe

pub const QUERY_GET_VOTES: &str = "SELECT video_id, score, round FROM votes WHERE video_id NOT IN (SELECT video_id FROM disqualified_videos);";

// ?1 - username
pub const QUERY_GET_USER_HASH: &str = "SELECT salt, password_hash FROM admins WHERE user = ?1;";


pub const QUERY_SETUP: &str = { "
    PRAGMA cache_size = 300000;
    PRAGMA page_size = 16384;
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = normal;

    CREATE TABLE IF NOT EXISTS videos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        youtube_id TEXT,
        uploader_username TEXT,
        category TEXT
    );
    
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip TEXT UNIQUE
    );
    
    CREATE TABLE IF NOT EXISTS votes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER,
        video_id INTEGER,
        score INTEGER,
        opponent_video_id INTEGER,
        round INTEGER,
        vote_time INTEGER NOT NULL,
        FOREIGN KEY (video_id) REFERENCES videos (id),
        FOREIGN KEY (opponent_video_id) REFERENCES videos (id),
        FOREIGN KEY (user_id) REFERENCES users (id)
    );

    CREATE TABLE IF NOT EXISTS ranked_finalist_votes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER,
        video_id INTEGER,
        rank INTEGER,
        vote_time INTEGER,
        FOREIGN KEY (user_id) REFERENCES users (id),
        FOREIGN KEY (video_id) REFERENCES videos (id)
    );

    CREATE TABLE IF NOT EXISTS active_votes (
        user_id INTEGER,
        video_id INTEGER,
        start_time INTEGER,
        FOREIGN KEY (user_id) REFERENCES users (id),
        FOREIGN KEY (video_id) REFERENCES videos (id)
    );
    
    CREATE TABLE IF NOT EXISTS culled_videos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        video_id INTEGER,
        phase INTEGER,
        FOREIGN KEY (video_id) REFERENCES videos (id)
    );
     
    CREATE TABLE IF NOT EXISTS disqualified_videos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        video_id INTEGER,
        reason TEXT,
        FOREIGN KEY (video_id) REFERENCES videos (id)
    );
           
    CREATE TABLE IF NOT EXISTS reports (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        video_id INTEGER,
        report_type TEXT,
        false_flag BOOL,
        timestamp INTEGER NOT NULL,
        addressed_by INTEGER,
        FOREIGN KEY (video_id) REFERENCES videos (id),
        FOREIGN KEY (addressed_by) REFERENCES admins (id)
    );

    CREATE TABLE IF NOT EXISTS admins (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user TEXT,
        salt TEXT,
        password_hash TEXT
    );
           
    INSERT INTO videos (youtube_id, uploader_username, category)
    SELECT * FROM (VALUES
    	('WRRC-Iw_OPg', 'user1', 'funny'),
    	('72eGw4H2Ka8', 'user2', 'funny'),
    	('4LilrtDfLP0', 'user3', 'funny'),
    	('uSlB4eznXoA', 'user4', 'funny'),
    	('i9bYnBb42oY', 'user5', 'funny'),
    	('lNfCvZl3sKw', 'user6', 'funny'),
    	('nz_BY7X44kc', 'user7', 'funny'),
    	('xrziHnudx3g', 'user8', 'funny'),
    	('4hpbK7V146A', 'user9', 'funny'),
    	('Ta_-UPND0_M', 'user10', 'funny'),
    	('JgJUbmGDc6k', 'user11', 'funny'),
    	('ttArr90NvWo', 'user12', 'funny'),
    	('mIpnpYsl-VY', 'user13', 'funny'),
    	('4LilrtDfLP0', 'user14', 'funny'),
    	('duAGuYeF7zY', 'user15', 'funny'),
    	('0pnwE_Oy5WI', 'user16', 'funny')
    )
    WHERE NOT EXISTS (SELECT * FROM videos);
" };
