Routes
---

* `/` `GET` main html page

* `/dashboard` `GET` dashboard html page

* `/vote` `GET` initiate a new vote

Expected query string:

``` javascript
{
   "c": Int  //voting category
}
```

Returned payload on 200:

``` javascript
{
   "videos": [
      {
         "youtube_id": String,
         "id": Int,
         "username": String        // optional
      }
   ], 
   "c": Int,                       //voting category
   "limit_active": Bool, 
   "stage": Int,
   "current_deadline": Int         // unix timestamp in seconds
}
```
<br/>

* `/vote` `POST` submit a vote

Expected payload:

``` javascript
{
   "vote": [Int]     // expects a list of any order, containing the ids
                     // expects the list to be the same size and
                     // contain the same ints as was given with the most recent /vote GET
}
```

Returns empty OK on 200.

<br/>

* `/admin/login` `POST` login to the admin dashboard
* `/server/config` `GET` get server parameters
* `/server/config` `POST` update server parameters
* `/server/tables` `GET` get database table data
* `/server/tables` `POST` update a cell within the db
