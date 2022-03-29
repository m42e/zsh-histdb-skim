#!/bin/bash
_histdb_query () {
    sqlite3 -batch -noheader -cmd ".timeout 1000" "test.db" "$@"
    [[ "$?" -ne 0 ]] && echo "error in $@"
}
echo "Create Tables"
_histdb_query <<-EOF
create table commands (id integer primary key autoincrement, argv text, unique(argv) on conflict ignore);
create table places   (id integer primary key autoincrement, host text, dir text, unique(host, dir) on conflict ignore);
create table history  (id integer primary key autoincrement,
                       session int,
                       command_id int references commands (id),
                       place_id int references places (id),
                       exit_status int,
                       start_time int,
                       duration int);
PRAGMA user_version = 2
EOF

echo "Create index"
_histdb_query<<-EOF
create index if not exists hist_time on history(start_time);
create index if not exists place_dir on places(dir);
create index if not exists place_host on places(host);
create index if not exists history_command_place on history(command_id, place_id);
PRAGMA journal_mode = WAL;
PRAGMA synchronous=normal;
EOF

insert_data(){
  cmd=$1
  pwd=$2
  host=$3
  session=$4
  started=$5

  _histdb_query <<-EOF
    insert into commands (argv) values ('${cmd}');
    insert into places   (host, dir) values ('${host}', '${pwd}');
    insert into history
      (session, command_id, place_id, start_time)
    select
      ${session},
      commands.id,
      places.id,
      ${started}
    from
      commands, places
    where
      commands.argv = '${cmd}' and
      places.host = '${host}' and
      places.dir = '${pwd}'
    ;
EOF
}

echo "Inserting fake test data"
insert_data "echo second" "$(pwd)" "testhost" "4713" 1640995210
insert_data "echo first" "$(pwd)" "testhost" "4713" 1640995209
insert_data "echo second" "/home/someone/skripts" "testhost" "4711" $(date +%s)
insert_data "echo first" "/home/someone/skripts" "testhost" "4711" 1640995201

insert_data "echo second2" "/home/someone/skripts" "testhost" "4712" $(date +%s)
insert_data "echo first2" "/home/someone/skripts" "testhost" "4712" 1640995202

insert_data "echo tmp" "/tmp" "otherhost" "12" 1640995205
