# shounic-best-clips-rs

The voting website to be used for shounic's upcoming best clip of the year contest this winter. Loosely Based off of [GameCube762's](https://github.com/Gamecube762/ShounicBestClips) website. 

## TODO

- [ ] Voting frontend
- [ ] Frontend admin dashboard
- [x] server/config endpoint
- [ ] server/tables endpoint
- [x] Refactor state struct to not be so ass
- [ ] Re-write interactions with the database so it isn't so ass
- [x] Prevent voting round value from ever decreasing
- [x] Make server/tables endpoint target single rows only
- [ ] Figure out how tf raw submissions are going to be handled
- [x] Remove token_cache 
- [ ] Implement a new config parameter for controlling eliminations
  - [ ] Consider a staggered thread-based cull
  - [ ] Endpoint handling 
  - [ ] Database
  - [x] Implement automatic round increase
- [x] Remove old tally system
- [ ] Replace all text responses with json ones
- [ ] Implement the ability to disable voting
- [ ] Clean up unused code

