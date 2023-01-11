set client_encoding = 'UTF8';

create table users (
  email varchar primary key,
  external_id varchar not null,
  user_name varchar not null,
  registered_date date not null,
  updated_date date not null
);