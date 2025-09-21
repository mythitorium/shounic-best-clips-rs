# shounic-best-clips-rs

The voting website to be used for shounic's upcoming best clip of the year contest this winter. Loosely Based off of [GameCube762's](https://github.com/Gamecube762/ShounicBestClips) website. 

## TODO

- [ ] Voting frontend
   - [ ] Html
   - [ ] Js
- [ ] Frontend admin dashboard
- [x] server/config endpoint
- [x] server/tables endpoint
- [x] Refactor state struct to not be so ass
- [ ] Re-write interactions with the database so it isn't so ass
- [x] Prevent voting round value from ever decreasing
- [x] Make server/tables endpoint target single rows only
- [ ] Figure out how tf raw submissions are going to be handled
- [x] Remove token_cache 
- [x] Implement a new config parameter for controlling eliminations
  - [x] Endpoint handling 
  - [x] Database
  - [x] Implement automatic round increase
- [x] Remove old tally system
- [x] Replace all text responses with json ones
- [x] Implement the ability to disable voting
- [x] Clean up unused code
- [x] More payload validation
- [x] Add a fake round counter that gets reported to the end user
- [x] Add the ability to send submission usernames to the frontend
- [x] Add vote cooldown
- [x] Add login cooldown

