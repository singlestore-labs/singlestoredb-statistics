To load the titanic dataset follow these steps:

1. cd into this directory
2. launch the mysql client with `--local-infile`
   ```bash
   # e.g.
   mysql -u root -p -h 127.0.0.1 --local-infile
   ```
2. select (or create) a database
   ```sql
   create database test;
   use test;
   ```
3. source schema.sql
   ```sql
   source schema.sql
   ```