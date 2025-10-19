Routes
---

* `/` `GET` main html page

* `/dashboard` `GET` dashboard html page

* `/vote` `GET` initiate a new vote

Expected query string:

``` json
{
   "c": int  //voting category
}
```

Returned payload on 200:

``` json
{
   "videos": [
      {
         "youtube_id": String,
         "id": int,
         "username": String        // optional
      }
   ], 
   "c": int,                       //voting category
   "limit_active": bool, 
   "stage": int,
   "current_deadline": int         // unix timestamp in seconds
}
```


* `/vote` `POST` submit a vote

Expected payload:

``` json
{
   "vote": [int]     // expects a list of any order, containing the ids
}
```

Returns empty OK on 200.


* `/admin/login` `POST` login to the admin dashboard
* `/server/config` `GET` get server parameters
* `/server/config` `POST` update server parameters
* `/server/tables` `GET` get database table data
* `/server/tables` `POST` update a cell within the db