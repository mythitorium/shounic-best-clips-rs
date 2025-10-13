//
// sql.rs
//
// Defining the sql queries and schema is so much easier in it's own file
//
// Param descriptions are for debugging/documentation
//

// ?1 - ip address
pub const QUERY_LOG_USER: &str = "INSERT OR IGNORE INTO users(ip, vote_banned, report_banned) VALUES (?1, 0, 0)";

// ?1 - ip address
pub const QUERY_GET_USER_ID: &str = "SELECT id, vote_banned, report_banned FROM users WHERE ip = ?1";

// ?1 - category being filtered for
// ?2 - the amount
pub const QUERY_GET_NEW_VOTABLE_VIDEOS: &str = "
    SELECT id, youtube_id, uploader_username FROM videos WHERE is_eliminated = 0 AND is_disqualified = 0 AND category = ?1 ORDER BY random() LIMIT ?2
";

// ?1 - user id
// ?2 - winning video id
// ?3 - score
// ?4 - opponent_video_id
// ?5 - voting round
// ?6 - vote time in unix
pub const QUERY_VOTE: &str = "INSERT INTO votes(user_id, video_id, score, opponent_video_id, round, vote_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

// ?1 - user id
// ?2 - video 1
// ?3 - start time in unix
// ?4 - category
pub const QUERY_SET_ACTIVE_VOTE: &str = "INSERT INTO active_votes(user_id, video_id, start_time, category) VALUES (?1, ?2, ?3, ?4)";

// ?1 - user id
pub const QUERY_CLEAR_ACTIVE_VOTES: &str = "DELETE FROM active_votes WHERE user_id = ?1";

// ?1 - user id
pub const QUERY_GET_ACTIVE_VOTE_VIDEOS: &str = "SELECT id, category, start_time FROM videos JOIN active_votes ON active_votes.video_id = videos.id WHERE active_votes.user_id = ?1;";

pub const QUERY_GET_VOTES_THIS_ROUND: &str = "SELECT user_id, videos.category FROM votes JOIN videos ON videos.id = votes.video_id WHERE round = ?1 GROUP BY user_id";

// ?1 - username
pub const QUERY_GET_USER_HASH: &str = "SELECT password_hash FROM admins WHERE user = ?1;";

// ?1 - category
// ?2 - round
// ?3 - category
// ?4 - elimination threshold
pub const QUERY_ELIMINATE_VIDEOS: &str = "
	UPDATE videos 
    SET is_eliminated = 1 
    WHERE category = ?1 AND id NOT IN (
        SELECT video_id 
        FROM votes 
		JOIN videos ON videos.id = votes.video_id
        WHERE 
            round = ?2 
            AND videos.category = ?3 
            AND opponent_video_id NOT IN (
                SELECT id FROM videos WHERE is_disqualified = 1
            )
            AND votes.user_id NOT IN (
                SELECT id FROM users WHERE vote_banned = 0
            )
        GROUP BY video_id
        ORDER BY AVG(score) DESC
		LIMIT ?4
    );
"; 

// ?1 round
// ?2 - category
// ?3 - amount
// ?4 - offset
pub const QUERY_FRONTEND_GET_RANKING_DATA: &str = "
    SELECT
        video_id,
        videos.youtube_id,
        videos.uploader_username,
        videos.is_eliminated,
        videos.is_disqualified,
        AVG(score) AS avg_score,
        COUNT(score) as total_votes 
    FROM votes
    JOIN videos ON videos.id = votes.video_id
    WHERE 
        votes.round = ?1 
        AND votes.category = ?2
        AND votes.opponent_video_id NOT IN (
            SELECT id FROM videos WHERE is_disqualified = 1
        )
        AND votes.user_id NOT IN (
            SELECT id FROM users WHERE vote_banned = 0
        )
    GROUP BY video_id
    ORDER BY avg_score DESC
    LIMIT ?3 OFFSET ?4
";

// ?1 - amount
// ?2 - offset
pub const QUERY_FRONTEND_GET_VIDEO_DATA: &str = "
    SELECT
        id,
        youtube_id,
        uploader_username,
        is_eliminated,
        is_disqualified
    FROM videos
    LIMIT ?3 OFFSET ?4
";

// ?1 - amount
// ?2 - offset
pub const QUERY_FRONTEND_GET_USER_DATA: &str = "SELECT * FROM users LIMIT ?1 OFFSET ?2";

// ?1 - amount
// ?2 - offset
pub const QUERY_FRONTEND_GET_REPORT_DATA: &str = "
    SELECT id, reporter, video_id, videos.youtube_id, videos.is_disqualified, timestamp, resolved 
    FROM reports 
    JOIN videos ON reports.video_id = videos.id
    LIMIT ?1 OFFSET ?2
";

// ?1 - what to set it to
// ?2 - id
pub const QUERY_DISQUALIFY_VIDEO: &str = "UPDATE videos SET is_disqualified = ?1 WHERE id = ?2";

// ?1 - what to set it to
// ?2 - id
pub const QUERY_VOTE_BAN_USER: &str = "UPDATE users SET vote_banned = ?1 WHERE id = ?2";

// ?1 - what to set it to
// ?2 - id
pub const QUERY_REPORT_BAN_USER: &str = "UPDATE users SET report_banned = ?! WHERE id = ?2";
 
// ?1 - what to set it to
// ?2 - id
pub const QUERY_MARK_REPORT_RESOLVED: &str = "UPDATE reports SET resolved = ?1 WHERE id = ?2";


pub const QUERY_SETUP: &str = { "
    PRAGMA cache_size = 300000;
    PRAGMA page_size = 16384;
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = normal;

    CREATE TABLE IF NOT EXISTS videos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        youtube_id TEXT,
        uploader_username TEXT,
        category INTEGER,
        is_eliminated INTEGER DEFAULT 0,
        is_disqualified INTEGER DEFAULT 0
    );
    
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        vote_banned INTEGER,
        report_banned INTEGER,
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

    CREATE TABLE IF NOT EXISTS active_votes (
        user_id INTEGER,
        video_id INTEGER,
        start_time INTEGER,
        category INTEGER,
        FOREIGN KEY (user_id) REFERENCES users (id),
        FOREIGN KEY (video_id) REFERENCES videos (id)
    );
           
    CREATE TABLE IF NOT EXISTS reports (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        reporter INTEGER,
        video_id INTEGER,
        timestamp INTEGER NOT NULL,
        resolved INTEGER,
        FOREIGN KEY (video_id) REFERENCES videos (id),
        FOREIGN KEY (reporter) REFERENCES users (id)
    );

    CREATE TABLE IF NOT EXISTS admins (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user TEXT,
        salt TEXT,
        password_hash TEXT
    );
" };


pub const QUERY_INSERT_PLACEHOLDER_VIDEOS: &str = "
    INSERT INTO videos (youtube_id, uploader_username, category, is_eliminated, is_disqualified)
    SELECT * FROM (VALUES
    	('WRRC-Iw_OPg', 'user1', 1, 0, 0),
    	('72eGw4H2Ka8', 'user2', 1, 0, 0),
    	('4LilrtDfLP0', 'user3', 1, 0, 0),
    	('uSlB4eznXoA', 'user4', 1, 0, 0),
    	('i9bYnBb42oY', 'user5', 1, 0, 0),
    	('lNfCvZl3sKw', 'user6', 1, 0, 0),
    	('nz_BY7X44kc', 'user7', 1, 0, 0),
    	('xrziHnudx3g', 'user8', 1, 0, 0),
    	('4hpbK7V146A', 'user9', 1, 0, 0),
    	('Ta_-UPND0_M', 'user10', 2, 0, 0),
    	('JgJUbmGDc6k', 'user11', 2, 0, 0),
    	('ttArr90NvWo', 'user12', 2, 0, 0),
    	('mIpnpYsl-VY', 'user13', 2, 0, 0),
    	('4LilrtDfLP0', 'user14', 2, 0, 0),
    	('duAGuYeF7zY', 'user15', 2, 0, 0),
    	('0pnwE_Oy5WI', 'user16', 2, 0, 0)
    )
    WHERE NOT EXISTS (SELECT * FROM videos);
";


pub const QUERY_INSERT_ROUND_OF_FAKE_VOTES: &str = "
    INSERT INTO votes(user_id, video_id, score, opponent_video_id, round, vote_time) VALUES
        (1, 1,  0, 9, 1, 0),
        (1, 2,  1, 8, 1, 0),
        (1, 3,  0, 7, 1, 0),
        (1, 4,  0, 6, 1, 0),
        (1, 5,  0, 4, 1, 0),
        (1, 4,  1, 5, 1, 0),
        (1, 6,  1, 4, 1, 0),
        (1, 7,  1, 3, 1, 0),
        (1, 8,  0, 2, 1, 0),
        (1, 9,  1, 1, 1, 0),
        (1, 10, 0, 16, 1, 0),
        (1, 11, 1, 15, 1, 0),
        (1, 12, 1, 14, 1, 0),
        (1, 12, 0, 13, 1, 0),
        (1, 13, 1, 12, 1, 0),
        (1, 14, 0, 12, 1, 0),
        (1, 15, 0, 11, 1, 0),
        (1, 16, 1, 10, 1, 0);
";
