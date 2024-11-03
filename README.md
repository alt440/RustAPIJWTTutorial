# RustAPIJWTTutorial
Rust API tutorial with the use of JWT tokens

## Get Optimized Executable
```
cargo build --release
``` 

## To Easily Update All Dependencies
```
cargo update
```

## Configuring your DB
To configure your database, first go to: https://www.sqlite.org/download.html

Then, unzip the zip file somewhere on your PC. Add the path where the content was unzipped to your environment variables in Windows.

Once this is done, open a console. Verify that you have it installed (and that it is working OK with the console):
```bash
sqlite3 --version
```

Then create a new database:
```bash
sqlite3 users.db
```

This will show the sqlite prompt. In there, execute the command:
```sql
CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, username TEXT, password TEXT, roles TEXT);
```

To exit the prompt, type '.' (the period character alone)

## Modules
You can have a folder name for a module (with its own mod.rs file, which represents the code for that module). Any files in the same folder with a different name will be considered as submodules.

Otherwise, you can have files at the same level as main.rs that you can import using only 'mod'