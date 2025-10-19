ShounicBestClips
===

Voting website for shounic's best clip contest. Rust port of and successor to [ShounicBestClips](https://github.com/Gamecube762/ShounicBestClips)

Setup
---

* Install rust
* Install cmake and clang (this project uses c-based dependencies)
* `git clone` the project
* `cargo run` to run the project on port `8080`


Routes
---

* `/` `GET` main html page

* `/dashboard` `GET` dashboard html page

* `/vote` `GET` initiate a new vote

Expected query string:

```
{
   c: int  //voting category
}
```

Returned payload on 200:

```
{
   videos: [
      {
         youtube_id: String,
         id: int,
         username: String        // optional
      }
   ], 
   c: int,                       //voting category
   limit_active: bool, 
   stage: int,
   current_deadline: int         // unix timestamp in seconds
}
```

* `/vote` `POST` submit a vote

Expected payload:

```
{
   
}
```

* `/admin/login` `POST` login to the admin dashboard
* `/server/config` `GET` get server parameters
* `/server/config` `POST` update server parameters
* `/server/tables` `GET` get database table data
* `/server/tables` `POST` update a cell within the db

TODO
---

- [x] Voting frontend
   - [x] Html
   - [x] Js
- [ ] Frontend admin dashboard
   - [x] Config
   - [ ] Table Dashboard
   - [ ] Upload
- [x] Backend

